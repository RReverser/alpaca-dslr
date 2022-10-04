import openapi from '@readme/openapi-parser';
import { readFile, writeFile } from 'fs/promises';
import { render } from 'ejs';
import { execFileSync } from 'child_process';
import { toSnakeCase, toPascalCase } from 'js-convert-case';

let [api, template] = await Promise.all([
  openapi.dereference('./AlpacaDeviceAPI_v1.yaml'),
  readFile('./server.ejs', 'utf-8')
]);

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

let rendered = render(
  template,
  {
    api,
    path2id,
    toTypeName: toPascalCase,
    toPropName: name => {
      let converted = toSnakeCase(name);
      if (converted === 'type') {
        converted += '_';
      }
      return converted;
    },
    getTypeDescription: function getTypeDescription(prop) {
      if ('description' in prop) {
        return prop.description;
      }
      if (prop.type === 'array') {
        return getTypeDescription(prop.items);
      }
      return '';
    },
    handleType: function handleType(prop) {
      switch (prop.type) {
        case 'integer':
          switch (prop.format) {
            case 'uint32':
              return 'u32';
            case 'int32':
              return 'i32';
          }
          break;
        case 'array':
          return `Vec<${handleType(prop.items)}>`;
        case 'number':
          return 'f64';
        case 'string':
          return 'String';
        case 'boolean':
          return 'bool';
      }
      console.warn(`Unhandled property type`, prop);
      return '()';
    },
    handleOptType: function (prop, required) {
      let type = this.handleType(prop);
      return required ? type : `Option<${type}>`;
    },
    getRequestBody: function getRequestBody(method) {
      if (!method.requestBody) {
        return;
      }
      let { content } = method.requestBody;
      let keys = Object.keys(content || {});
      if (!content || !keys.length) {
        console.warn('Request body without content', method);
        return;
      }
      if (keys.length > 1) {
        console.warn('Request body with multiple content types', method);
        return;
      }
      let contentType = keys[0],
        type;
      switch (contentType) {
        case 'application/json':
          type = 'Json';
          break;
        case 'application/x-www-form-urlencoded':
          type = 'Form';
          break;
        default:
          console.warn('Request body with unsupported content type', method);
          return;
      }
      let { schema } = content[contentType];
      if (!schema || schema.type !== 'object') {
        console.warn('Request body with unsupported schema', method);
        return;
      }
      return { type, schema };
    }
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
