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

      let name = `${typeId}${toPascalCase(group)}`;
      api.components!.schemas![name] = params;
      let existingVia = (params as any)['x-request'];
      if (existingVia) {
        assert.equal(via, existingVia);
      }
      (params as any)['x-request'] = via;
      return { $ref: `#/components/schemas/${name}` };
    }
  );

  let requestBody = resolveMaybeRef(operation.requestBody);
  if (requestBody) {
    let { content, schema } = getContent(
      requestBody,
      'application/x-www-form-urlencoded'
    );

    if (!isRef(content.schema)) {
      let via = 'Form';
      let name = `${typeId}Request`;
      api.components!.schemas![name] = schema;
      let existingVia = (schema as any)['x-request'];
      if (existingVia) {
        assert.equal(via, existingVia);
      }
      (schema as any)['x-request'] = via;
      content.schema = { $ref: `#/components/schemas/${name}` };
    }

    operation['x-parameterSchemas'].push(content.schema);
  }

  let { 200: successfulResponse, ...errorResponses } = operation.responses;

  assert.ok(
    successfulResponse,
    `Missing successful response for ${method} ${path}`
  );

  successfulResponse = resolveMaybeRef(successfulResponse);

  let { content, schema } = getContent(successfulResponse, 'application/json');

  if (!isRef(content.schema)) {
    let name = `${typeId}Response`;
    api.components!.schemas![name] = schema;
    content.schema = {
      $ref: `#/components/schemas/${name}`
    };
  }

  (schema as any)['x-response'] = true;
  operation['x-response'] = content.schema;

  let errorResponseShape: Partial<OpenAPIV3.ResponseObject> = {
    content: {
      'text/plain': {
        schema: {
          type: 'string'
        }
      }
    }
  };
  let errorResponsesWithoutDescriptions = JSON.parse(
    JSON.stringify(errorResponses, (k, v) =>
      k === 'description' ? undefined : v
    )
  );
  assert.deepEqual(errorResponsesWithoutDescriptions, {
    400: errorResponseShape,
    500: errorResponseShape
  });
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
