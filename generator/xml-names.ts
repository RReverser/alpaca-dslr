import { readFile } from 'fs/promises';
import * as assert from 'assert/strict';

class CanonicalDevice {
  private _methods: Record<string, string> = {};

  constructor(public readonly name: string) {}

  registerMethod(method: string) {
    this._methods[method.toLowerCase()] = method;
  }

  getMethod(subPath: string) {
    let name = this._methods[subPath];
    assert.ok(
      name,
      `Couldn't find canonical name for ${this.name}::${subPath}`
    );
    return name;
  }
}

class CanonicalDevices {
  private _devices: Record<string, CanonicalDevice> = {};

  registerDevice(name: string) {
    return (this._devices[name.toLowerCase()] ??= new CanonicalDevice(name));
  }

  getDevice(path: string) {
    let device = this._devices[path];
    assert.ok(device, `Couldn't find canonical device for ${path}`);
    return device;
  }
}

export async function getCanonicalNames() {
  let xml = await readFile('./ascom.alpaca.simulators.xml', 'utf-8');

  let canonical = new CanonicalDevices();

  for (let [, device, method] of xml.matchAll(
    /M:ASCOM\.Alpaca\.Simulators\.(\w+?)(?:Controller)?\.(\w+)\(/g
  )) {
    canonical.registerDevice(device).registerMethod(method);
  }

  let generic = canonical.registerDevice('{device_type}');
  for (let [, method] of xml.matchAll(/M:Alpaca\.AlpacaController\.(\w+)\(/g)) {
    generic.registerMethod(method);
  }

  return canonical;
}
