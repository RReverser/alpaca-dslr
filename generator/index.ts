import openapi from '@readme/openapi-parser';
import { readFile, writeFile } from 'fs/promises';
import { render } from 'ejs';
import { execFileSync } from 'child_process';
import { toSnakeCase, toPascalCase } from 'js-convert-case';
import { OpenAPIV3 } from 'openapi-types';
import * as assert from 'assert/strict';
import { inspect } from 'util';

interface MethodExtension {
  'x-parameterSchemas'?: OpenAPIV3.ReferenceObject[];
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

function withoutDescriptions<T>(obj: T): T {
  return JSON.parse(
    JSON.stringify(obj, (k, v) => (k === 'description' ? undefined : v))
  );
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

  operation['x-parameterSchemas'] = Object.entries(groupedParams).map(
    ([group, params]) => {
      let via;
      switch (group) {
        case 'path':
          via = 'Path';
          break;
        case 'query':
          via = 'Query';
          break;
        default:
          throw new Error(`Unsupported parameter group: ${group}`);
      }

      setXKind(params, via);
      return registerSchema(`${typeId}${toPascalCase(group)}`, params);
    }
  );

  let requestBody = resolveMaybeRef(operation.requestBody);
  if (requestBody) {
    let { content, schema } = getContent(
      requestBody,
      'application/x-www-form-urlencoded'
    );

    if (!isRef(content.schema)) {
      content.schema = registerSchema(`${typeId}Request`, schema);
    }

    setXKind(schema, 'Form');
    operation['x-parameterSchemas'].push(content.schema);
  }

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
  assert.deepEqual(withoutDescriptions(errorResponses), {
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

for (let schema of Object.values(api.components!.schemas!)) {
  schema = resolveMaybeRef(schema);

  switch ((schema as any)['x-kind']) {
    case 'Response': {
      let {
        ClientTransactionID,
        ServerTransactionID,
        ErrorNumber,
        ErrorMessage,
        ...otherProperties
      } = schema.properties!;
      assert.deepEqual(
        withoutDescriptions({
          ClientTransactionID,
          ServerTransactionID,
          ErrorNumber,
          ErrorMessage
        }),
        {
          ClientTransactionID: {
            type: 'integer',
            format: 'uint32',
            minimum: 0,
            maximum: 4294967295
          },
          ServerTransactionID: {
            type: 'integer',
            format: 'uint32',
            minimum: 0,
            maximum: 4294967295
          },
          ErrorNumber: {
            type: 'integer',
            format: 'int32',
            minimum: -2147483648,
            maximum: 2147483647
          },
          ErrorMessage: {
            type: 'string'
          }
        }
      );

      schema.properties = otherProperties;
      break;
    }
  }
}

let rendered = render(
  template,
  {
    api,
    refs,
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
