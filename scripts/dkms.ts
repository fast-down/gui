// https://github.com/koishi-registry/cyan-koishi-registry-private/blob/unstable/tools/dkms.ts
// Copyright Cyan Changes 2025. All rights reserved

import { exists, expandGlob } from "jsr:@std/fs";
import { crypto } from "jsr:@std/crypto/crypto";
import { Buffer } from "node:buffer";
import * as _ from "jsr:@radashi-org/radashi";
import { dirname, join, relative, resolve } from "jsr:@std/path";
import { type } from "npm:arktype";
import { makeArray } from "npm:cosmokit";
import JSON5 from "npm:json5";
import consola from "npm:consola";
import $ from "jsr:@david/dax";

export const dkmsSchema = type({
  name: "string",
  watch: type("string | string[]").pipe((x) => makeArray(x)),
  script: "string",
});
export const dkmsConf = type.or(type(dkmsSchema, "[]")).pipe((x) =>
  makeArray(x)
);

async function buildTask(root: string, shell_: string) {
  const shell = shell_
    .replace(/exec@([\w-_]+)/, `${Deno.execPath()} run -A npm:$1`);

  console.info(`%c$ %s`, "color: gray", shell);
  return $`sh -c ${shell}`
    .stderr("inherit")
    .cwd(root)
    .env("PATH", `${Deno.env.get("PATH")}`)
    .noThrow();
}

/// Returns: if two version are differ
export function compareVersions(
  a?: Record<string, string>,
  b?: Record<string, string>,
) {
  if (!_.isPlainObject(a)) return true;
  if (!_.isPlainObject(b)) return true;
  const aKeys = _.keys(a);
  const bKeys = _.keys(b);
  let diff = false;
  for (const key of aKeys) {
    if (!bKeys.includes(key)) {
      diff = true;
      console.log("%c - %s %c@ %s", "color: red;", key, "color: gray;", a[key]);
    } else if (!_.isEqual(a[key], b[key])) {
      diff = true;
      console.log(
        "%c * %s *",
        "color: yellow;",
        key,
      );
      console.log(
        "%c | %c-%c@%c%s",
        "color: yellow;",
        "color: red;",
        "color: yellow;",
        "color: blue;",
        a[key],
      );
      console.log(
        "%c | %c+%c@%c%s",
        "color: yellow;",
        "color: green;",
        "color: yellow;",
        "color: blue;",
        b[key],
      );
    }
  }
  for (const key of bKeys) {
    if (!aKeys.includes(key)) {
      diff = true;
      console.log(
        "%c + %s %c@ %s",
        "color: green;",
        key,
        "color: gray;",
        b[key],
      );
    }
  }
  return diff;
}

const kRecover = Symbol("kra.dkms.error.recoverable")

export function recoverable<T>(err: T): T {
  return Object.defineProperty(err, kRecover, { value: true });
}


export function isRecoverable<T>(err: T): bool {
  if (typeof err === 'object') return !!Reflect.get(err, kRecover)
  return false
}

export function maybeRecover(reason: unknown) {
  if (isRecoverable(reason)) {
    // consola.warn(reason)
    return
  }
  throw reason
}

export async function buildIfNeeded(meta_dir: string) {
  const meta_path = await (async () => {
    const pj = join(meta_dir, "package.json");
    const dj = join(meta_dir, "deno.json");
    if (await exists(dj)) return dj;
    if (await exists(pj)) return pj;
    throw recoverable(new TypeError(`No package.json or deno.json found in ${meta_dir}`));
  })();

  const meta = JSON.parse(await Deno.readTextFile(meta_path));
  if (!meta.dkms) return false;

  const dkms = dkmsConf.assert(meta.dkms);

  const path = resolve(meta_dir, "./.dkms.state.json5");
  const original = await exists(path)
    .then((exist) =>
      exist
        ? Promise.try(async () =>
          JSON5.parse(
            await Deno.readTextFile(path),
          )
        )
        : {}
    )
    .catch(() => {});
  const stateFile = await Deno.open(path, { create: true, write: true });
  const encodeStream = new TextEncoderStream();
  const promise = encodeStream.readable.pipeTo(stateFile.writable);
  const writer = encodeStream.writable.getWriter();

  writer.write("{");

  const tasks: ({
    root: string;
    name: string;
    script: string;
  })[] = [];

  const state = {};

  for (const task of dkms) {
    const globs = task.watch.map((pattern) =>
      expandGlob(pattern, { root: meta_dir })
    );
    const paths = (await Promise.all(
      globs.map(async (glob) => {
        const entries = await Array.fromAsync(glob);
        return entries.map((entry) => entry.path);
      }),
    )).flat().sort();

    const hashes: Record<string, string> = Object.fromEntries(
      await Promise.all(
        paths.filter((path) => Deno.lstatSync(path).isFile).map(
          async (path) => {
            return [
              relative(meta_dir, path),
              Buffer.from(
                await crypto.subtle.digest("BLAKE3", await Deno.readFile(path)),
              ).toString("base64"),
            ];
          },
        ),
      ),
    );

    console.info(
      "%c[dkms] %c%s%c#%c%s",
      "color: yellow;",
      "color: blue;",
      relative(Deno.cwd(), meta_dir),
      "color: aqua;",
      "color: green;",
      task.name,
    );
    if (compareVersions(original?.[task.name], hashes)) {
      tasks.push({
        name: task.name,
        root: meta_dir,
        script: meta.scripts[task.script],
      });
    }
    writer.write(`${JSON.stringify(task.name)}: ${JSON.stringify(hashes)},`);
    state[task.name] = hashes;
  }

  writer.write("}");
  await writer.close();
  await promise;

  for (const task of tasks) {
    console.info(
      "%c[dkms] %c$%o",
      "color: yellow;",
      "color: gray;",
      task.script,
    );
    const result = await buildTask(task.root, task.script);
    const exitCode = result.code;
    if (exitCode === 0) {
      console.info(
        "%c[dkms] %c$%o %cexited successfully",
        "color: yellow;",
        "color: gray;",
        task.script,
        "color: reset;",
      );
    } else {
      console.warn(
        "%c[dkms] %c$%o %cexited with error %o",
        "color: yellow;",
        "color: gray;",
        task.script,
        "color: red;",
        exitCode
      );
      await Deno.writeTextFile(path, JSON5.stringify(original, void 0, 2));
      return true;
    }
  }
  await Deno.writeTextFile(path, JSON5.stringify(state, void 0, 2));

  return true;
}

export async function searchPackages(directory: string) {
  await Promise.all(
    await Array.fromAsync(Deno.readDir(directory))
      .then(
        (entries) =>
          entries.map((entry) => {
            if (!entry.isDirectory) return
            return buildIfNeeded(resolve(directory, entry.name)).catch(maybeRecover)
          }),
      ),
  );
}

if (import.meta.main) {
  console.info(
    "%c@@@ %c~~~~~~~~~%c-----%c~~~~~~~~~~~~%c @@@",
    "color: gray;",
    "color: cyan;",
    "color: blue;",
    "color: cyan;",
    "color: gray;",
  );
  console.info(
    "%c@@@ %cKra DKMS %c_nya_ %cBy My Side %c @@@",
    "color: gray;",
    "color: cyan;",
    "color: blue;",
    "color: cyan;",
    "color: gray;",
  );
  console.info(
    "%c@@@ %c~~~~~~~~~%c-----%c~~~~~~~~~~~~%c @@@",
    "color: gray;",
    "color: cyan;",
    "color: blue;",
    "color: cyan;",
    "color: gray;",
  );
  await searchPackages(".");
  // todo
}