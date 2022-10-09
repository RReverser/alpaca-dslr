import { OpenAPIV3 } from 'openapi-types';

let extraSchemas: Record<string, OpenAPIV3.NonArraySchemaObject> = {};

for (let kind of ['Path', 'Request', 'Response']) {
  extraSchemas[`Empty${kind}`] = {
    type: 'object',
    properties: {},
    // @ts-ignore
    'x-kind': kind
  };
}

export { extraSchemas };
