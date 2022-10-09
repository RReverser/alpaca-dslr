import {
  CSharpClass,
  CSharpEnum,
  CSharpFile,
  CSharpInterface,
  CSharpNamespace,
  FileParser
} from '@fluffy-spoon/csharp-parser';
import { readdir, readFile } from 'fs/promises';

export async function getCSharpItems() {
  let files = await readdir('elendil-devices');

  function getNamespace(obj: CSharpFile | CSharpNamespace, path: string) {
    for (let name of path.split('.')) {
      let ns = obj.namespaces.find(ns => ns.name === name);
      if (!ns) {
        throw new Error(`Namespace ${name} not found`);
      }
      obj = ns;
    }
    return obj;
  }

  let mergedNS: Record<string, CSharpClass | CSharpInterface | CSharpEnum> = {};

  await Promise.all(
    files.map(async filename => {
      let text = await readFile(`elendil-devices/${filename}`, 'utf8');
      let file = new FileParser(text).parseFile();
      let ns = getNamespace(file, 'ES.Ascom.Alpaca.Devices');
      for (let array of [ns.classes, ns.interfaces, ns.enums]) {
        for (let item of array) {
          (item as any).className = item.constructor.name;
          mergedNS[item.name.toLowerCase()] = item;
        }
      }
    })
  );

  // mergedNS = JSON.parse(
  //   JSON.stringify(mergedNS, (key, value) => {
  //     if (key === 'parent' || key === 'innerScopeText') {
  //       return undefined;
  //     }
  //     return value;
  //   })
  // );

  return mergedNS;
}
