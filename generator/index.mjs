import openapi from '@readme/openapi-parser';
import { readFile, writeFile } from 'fs/promises';
import { render } from 'ejs';
import { execFileSync } from 'child_process';
import { toSnakeCase, toPascalCase } from 'js-convert-case';

let [api, template] = await Promise.all([
  openapi.dereference('./AlpacaDeviceAPI_v1.yaml'),
  readFile('./server.ejs', 'utf-8')
]);

let rendered = render(
  template,
  {
    api,
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
