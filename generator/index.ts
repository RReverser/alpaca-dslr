import openapi from '@readme/openapi-parser';
import { readFile, writeFile } from 'fs/promises';
import { render } from 'ejs';
import { execFileSync } from 'child_process';
import { toSnakeCase, toPascalCase } from 'js-convert-case';
import { OpenAPIV3 } from 'openapi-types';
import * as assert from 'assert/strict';
import { inspect, isDeepStrictEqual } from 'util';
import { extraSchemas } from './extra-schemas.js';

interface MethodExtension {
  'x-path'?: OpenAPIV3.ReferenceObject;
  'x-request'?: OpenAPIV3.ReferenceObject;
  'x-requestKind'?: 'Query' | 'Form';
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

function cleanupSchema(obj: OpenAPIV3.SchemaObject) {
  switch (obj.type) {
    case 'array': {
      if (!isRef(obj.items)) {
        cleanupSchema(obj.items);
      }
      return;
    }
    case 'object': {
      for (let v of Object.values(obj.properties!)) {
        if (!isRef(v)) {
          cleanupSchema(v);
        }
      }
      return;
    }
    case 'string': {
      if (obj.default === '') {
        delete obj.default;
      }
      return;
    }
    case 'boolean': {
      if (obj.default === false) {
        delete obj.default;
      }
      return;
    }
    case 'integer': {
      obj.format ??= 'int32';
      let range = {
        int32: { min: -2147483648, max: 2147483647 },
        uint32: { min: 0, max: 4294967295 }
      }[obj.format];
      if (range) {
        if (obj.minimum === range.min) {
          delete obj.minimum;
        }
        if (obj.maximum === range.max) {
          delete obj.maximum;
        }
      }
      // fallthrough
    }
    case 'number': {
      if (obj.default === 0) {
        delete obj.default;
      }
      return;
    }
  }
}

function jsonWithoutOptFields(obj: any): string {
  return JSON.stringify(obj, (k, v) => (k === 'description' ? undefined : v));
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
    let schema = (groupedParams[param.in] ??= {
      type: 'object',
      properties: {}
    });
    schema.properties![param.name] = {
      description: param.description,
      ...param.schema
    };
    if (param.required) {
      (schema.required ??= []).push(param.name);
    }
  }

  let typeId = toPascalCase(id);

  let { path: pathParams, query: queryParams, ...other } = groupedParams;
  assert.deepEqual(other, {});

  assert.ok(pathParams, `Missing path parameters for ${id}`);
  setXKind(pathParams, 'Path');
  let pathParamsRef = registerSchema(`${typeId}Path`, pathParams);

  let requestBody = resolveMaybeRef(operation.requestBody);

  let requestParams;

  if (method === 'get') {
    assert.ok(queryParams, `Missing query parameters for ${id}`);
    assert.ok(!requestBody, `Unexpected request body for ${id}`);

    setXKind(queryParams, 'Request');
    requestParams = {
      ref: registerSchema(`${typeId}Request`, queryParams),
      kind: 'Query' as const
    };
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

    setXKind(schema, 'Request');
    requestParams = {
      ref: content.schema,
      kind: 'Form' as const
    };
  }

  operation['x-path'] = pathParamsRef;
  operation['x-request'] = requestParams.ref;
  operation['x-requestKind'] = requestParams.kind;

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

let extraSchemasWithoutOptFields = Object.fromEntries(
  Object.entries(extraSchemas).map(([name, schema]) => {
    return [jsonWithoutOptFields(schema), name];
  })
);

for (let [schemaName, schema] of Object.entries(api.components!.schemas!)) {
  schema = resolveMaybeRef(schema);
  cleanupSchema(schema);

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
    case 'Request': {
      let { ClientID, ClientTransactionID, ...otherProperties } =
        schema.properties!;

      // These defaults are missing in some definitions.
      // Ignore them for comparison purposes.
      delete (ClientID as OpenAPIV3.SchemaObject).default;
      delete (ClientTransactionID as OpenAPIV3.SchemaObject).default;

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

  let replacementName =
    extraSchemasWithoutOptFields[jsonWithoutOptFields(schema)];

  if (replacementName) {
    delete api.components!.schemas![schemaName];

    refReplacements[`#/components/schemas/${schemaName}`] = {
      $ref: `#/components/schemas/${replacementName}`
    };
  } else {
    let key = jsonWithoutOptFields(schema);
    groupCounts[key] ??= 0;
    groupCounts[key]++;
  }
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

try {
  execFileSync('rustfmt', [
    '+nightly',
    '--edition=2021',
    'AlpacaDeviceAPI_v1.rs'
  ]);
} catch {
  throw new Error('rustfmt failed');
}
