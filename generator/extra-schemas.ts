import { OpenAPIV3 } from 'openapi-types';

let extraSchemas: Record<string, OpenAPIV3.NonArraySchemaObject> = {
  DeviceNumberPath: {
    type: 'object',
    properties: {
      device_number: {
        type: 'integer',
        format: 'uint32',
        description:
          'Zero based device number as set on the server (0 to 4294967295)'
      }
    },
    required: ['device_number'],
    // @ts-ignore
    'x-kind': 'Path'
  },
  DeviceTypeAndNumberPath: {
    type: 'object',
    properties: {
      device_type: {
        type: 'string',
        default: 'telescope',
        pattern: '^[a-z]*$',
        description:
          'One of the recognised ASCOM device types e.g. telescope (must be lower case)'
      },
      device_number: {
        type: 'integer',
        format: 'uint32',
        description:
          'Zero based device number as set on the server (0 to 4294967295)'
      }
    },
    required: ['device_type', 'device_number'],
    // @ts-ignore
    'x-kind': 'Path'
  }
};

for (let kind of ['Path', 'Request', 'Response']) {
  extraSchemas[`Empty${kind}`] = {
    type: 'object',
    properties: {},
    // @ts-ignore
    'x-kind': kind
  };
}

export { extraSchemas };
