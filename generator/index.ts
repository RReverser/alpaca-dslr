import openapi from '@readme/openapi-parser';
import { readFile, writeFile } from 'fs/promises';
import { render } from 'ejs';
import { execFileSync } from 'child_process';
import { toSnakeCase, toPascalCase } from 'js-convert-case';
import { OpenAPIV3 } from 'openapi-types';
import * as assert from 'assert/strict';
import { inspect } from 'util';

interface MethodExtension {
  'x-path'?: OpenAPIV3.ReferenceObject;
  'x-body'?: OpenAPIV3.ReferenceObject;
  'x-response'?: OpenAPIV3.ReferenceObject;
}

let api = (await openapi.parse(
  './AlpacaDeviceAPI_v1.yaml'
)) as OpenAPIV3.Document<MethodExtension>;
let refs = await openapi.resolve(api);
let template = await readFile('./server.ejs', 'utf-8');

let path2id = Object.fromEntries(
  Object.keys(api.paths || {}).map(path => [
    path,
    path
      .split('/')
      .slice(1)
      .filter(x => !/^\{.*\}$/.test(x))
      .join('_')
  ])
);

function* ops() {
  for (let [path, methods = {}] of Object.entries(api.paths)) {
    let pathId = path2id[path];
    for (let method of Object.values(OpenAPIV3.HttpMethods)) {
      let operation = methods[method];
      if (operation) {
        yield { path, method, id: `${method}_${pathId}`, operation };
      }
    }
  }
}

function isRef(maybeRef: any): maybeRef is OpenAPIV3.ReferenceObject {
  return maybeRef != null && '$ref' in maybeRef;
}

function resolveMaybeRef<T>(ref: T | OpenAPIV3.ReferenceObject): T {
  if (isRef(ref)) {
    return refs.get(ref.$ref);
  }
  return ref;
}

function getContent(
  owner: OpenAPIV3.RequestBodyObject | OpenAPIV3.ResponseObject,
  contentType: string
) {
  let { content = {} } = owner;
  let keys = Object.keys(content);
  assert.deepEqual(keys, [contentType], `Unexpected content types: ${keys}`);
  let { schema } = content[contentType];
  schema = resolveMaybeRef(schema);
  assert.equal(schema?.type, 'object', 'Content is not an object');
  return { schema: schema, content: content[contentType] };
}

function jsonWithoutOptFields(obj: any): string {
  return JSON.stringify(obj, (k, v) => {
    switch (k) {
      case 'description':
      case 'minimum':
      case 'maximum':
      case 'default':
        return;
      default:
        return v;
    }
  });
}

function withoutOptFields<T>(obj: T): T {
  return JSON.parse(jsonWithoutOptFields(obj));
}

function setXKind(schema: OpenAPIV3.SchemaObject, kind: string) {
  let prev = (schema as any)['x-kind'];
  if (prev) {
    assert.equal(prev, kind, `Conflicting x-kind: ${prev} vs ${kind}`);
  }
  (schema as any)['x-kind'] = kind;
}

function registerSchema(
  name: string,
  schema: OpenAPIV3.SchemaObject
): OpenAPIV3.ReferenceObject {
  api.components!.schemas![name] = schema;
  return { $ref: `#/components/schemas/${name}` };
}

// Extract path parameters, query parameters and bodies that are not in api.components yet.
// Add all of them to api.components, and replace the originals with $ref references.
for (let { method, path, id, operation } of ops()) {
  let groupedParams: Record<string, OpenAPIV3.NonArraySchemaObject> = {};
  for (let param of operation.parameters || []) {
    param = resolveMaybeRef(param);
    (groupedParams[param.in] ??= {
      type: 'object',
      properties: {}
    }).properties![param.name] = {
      description: param.description,
      ...param.schema
    };
  }

  let typeId = toPascalCase(id);

  let { path: pathParams, query: queryParams, ...other } = groupedParams;
  assert.deepEqual(other, {});

  assert.ok(pathParams, `Missing path parameters for ${id}`);
  setXKind(pathParams, 'Path');
  let pathParamsRef = registerSchema(`${typeId}Path`, pathParams);

  let requestBody = resolveMaybeRef(operation.requestBody);

  let bodyParamsRef;
  if (method === 'get') {
    assert.ok(queryParams, `Missing query parameters for ${id}`);
    assert.ok(!requestBody, `Unexpected request body for ${id}`);

    setXKind(queryParams, 'Query');
    bodyParamsRef = registerSchema(`${typeId}Query`, queryParams);
  } else {
    assert.ok(!queryParams, `Unexpected query parameters for ${id}`);
    assert.ok(requestBody, `Missing request body for ${id}`);

    let { content, schema } = getContent(
      requestBody,
      'application/x-www-form-urlencoded'
    );

    if (!isRef(content.schema)) {
      content.schema = registerSchema(`${typeId}Request`, schema);
    }

    setXKind(schema, 'Form');
    bodyParamsRef = content.schema;
  }

  operation['x-path'] = pathParamsRef;
  operation['x-body'] = bodyParamsRef;

  let { 200: successfulResponse, ...errorResponses } = operation.responses;

  assert.ok(
    successfulResponse,
    `Missing successful response for ${method} ${path}`
  );

  let errorResponseShape: Partial<OpenAPIV3.ResponseObject> = {
    content: {
      'text/plain': {
        schema: {
          type: 'string'
        }
      }
    }
  };
  assert.deepEqual(withoutOptFields(errorResponses), {
    400: errorResponseShape,
    500: errorResponseShape
  });

  successfulResponse = resolveMaybeRef(successfulResponse);

  let { content, schema } = getContent(successfulResponse, 'application/json');

  if (!isRef(content.schema)) {
    content.schema = registerSchema(`${typeId}Response`, schema);
  }

  setXKind(schema, 'Response');
}

let refReplacements: Record<string, OpenAPIV3.ReferenceObject> = {};
let groupCounts: Record<string, number> = {};
let extraSchemas: Record<string, OpenAPIV3.NonArraySchemaObject> = {};

for (let [schemaName, schema] of Object.entries(api.components!.schemas!)) {
  schema = resolveMaybeRef(schema);
  let kind = (schema as any)['x-kind'];

  switch (kind) {
    case 'Response': {
      let {
        ClientTransactionID,
        ServerTransactionID,
        ErrorNumber,
        ErrorMessage,
        ...otherProperties
      } = schema.properties!;

      assert.deepEqual(
        withoutOptFields({
          ClientTransactionID,
          ServerTransactionID,
          ErrorNumber,
          ErrorMessage
        }),
        {
          ClientTransactionID: {
            type: 'integer',
            format: 'uint32'
          },
          ServerTransactionID: {
            type: 'integer',
            format: 'uint32'
          },
          ErrorNumber: {
            type: 'integer',
            format: 'int32'
          },
          ErrorMessage: {
            type: 'string'
          }
        },
        `Missing response properties in ${schemaName}`
      );

      schema.properties = otherProperties;
      break;
    }
    case 'Form':
    case 'Query': {
      let { ClientID, ClientTransactionID, ...otherProperties } =
        schema.properties!;

      assert.deepEqual(
        withoutOptFields({ ClientID, ClientTransactionID }),
        {
          ClientID: {
            type: 'integer',
            format: 'uint32'
          },
          ClientTransactionID: {
            type: 'integer',
            format: 'uint32'
          }
        },
        `Missing request properties in ${schemaName}`
      );

      schema.properties = otherProperties;
      break;
    }
  }

  if (
    schema.type === 'object' &&
    Object.keys(schema.properties!).length === 0
  ) {
    delete api.components!.schemas![schemaName];

    let name = `Empty${kind || 'Object'}`;

    extraSchemas[name] ??= {
      type: 'object',
      properties: {},
      description: `Empty ${name}`,
      // @ts-ignore
      'x-kind': kind
    };

    refReplacements[`#/components/schemas/${schemaName}`] = {
      $ref: `#/components/schemas/${name}`
    };
  }

  let key = jsonWithoutOptFields(schema);
  groupCounts[key] ??= 0;
  groupCounts[key]++;
}

Object.assign(api.components!.schemas!, extraSchemas);

Object.entries(groupCounts)
  .filter(([_, count]) => count > 1)
  .sort(([_, count1], [__, count2]) => count2 - count1)
  .forEach(([json, count]) => {
    console.warn(
      `Found ${count} schemas with the same shape:`,
      JSON.parse(json)
    );
  });

let rendered = render(
  template,
  {
    api,
    refs,
    refReplacements,
    ops,
    assert,
    dbg: (x: any) => inspect(x, { colors: true }),
    getContent,
    toPascalCase,
    toSnakeCase
  },
  {
    escape: x => x
  }
);

await writeFile('./AlpacaDeviceAPI_v1.rs', rendered);

execFileSync('rustfmt', [
  '+nightly',
  '--edition=2021',
  'AlpacaDeviceAPI_v1.rs'
]);
