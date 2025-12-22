import { r as ref, o as onUnmounted, c as computed, d as defineComponent, a as createElementBlock, b as openBlock, e as createVNode, T as Transition, w as withCtx, f as createBaseVNode, n as normalizeStyle, t as toDisplayString, g as createCommentVNode, h as createTextVNode } from './index-CkkE85_b.js';
import { _ as _export_sfc } from './_plugin-vue_export-helper-pcqpp-6-.js';

/******************************************************************************
Copyright (c) Microsoft Corporation.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
***************************************************************************** */
/* global Reflect, Promise, SuppressedError, Symbol, Iterator */


function __classPrivateFieldGet(receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
}

function __classPrivateFieldSet(receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
}

typeof SuppressedError === "function" ? SuppressedError : function (error, suppressed, message) {
    var e = new Error(message);
    return e.name = "SuppressedError", e.error = error, e.suppressed = suppressed, e;
};

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var _Channel_onmessage, _Channel_nextMessageIndex, _Channel_pendingMessages, _Channel_messageEndIndex, _Resource_rid;
/**
 * Invoke your custom commands.
 *
 * This package is also accessible with `window.__TAURI__.core` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */
/**
 * A key to be used to implement a special function
 * on your types that define how your type should be serialized
 * when passing across the IPC.
 * @example
 * Given a type in Rust that looks like this
 * ```rs
 * #[derive(serde::Serialize, serde::Deserialize)
 * enum UserId {
 *   String(String),
 *   Number(u32),
 * }
 * ```
 * `UserId::String("id")` would be serialized into `{ String: "id" }`
 * and so we need to pass the same structure back to Rust
 * ```ts
 * import { SERIALIZE_TO_IPC_FN } from "@tauri-apps/api/core"
 *
 * class UserIdString {
 *   id
 *   constructor(id) {
 *     this.id = id
 *   }
 *
 *   [SERIALIZE_TO_IPC_FN]() {
 *     return { String: this.id }
 *   }
 * }
 *
 * class UserIdNumber {
 *   id
 *   constructor(id) {
 *     this.id = id
 *   }
 *
 *   [SERIALIZE_TO_IPC_FN]() {
 *     return { Number: this.id }
 *   }
 * }
 *
 * type UserId = UserIdString | UserIdNumber
 * ```
 *
 */
// if this value changes, make sure to update it in:
// 1. ipc.js
// 2. process-ipc-message-fn.js
const SERIALIZE_TO_IPC_FN = '__TAURI_TO_IPC_KEY__';
/**
 * Stores the callback in a known location, and returns an identifier that can be passed to the backend.
 * The backend uses the identifier to `eval()` the callback.
 *
 * @return An unique identifier associated with the callback function.
 *
 * @since 1.0.0
 */
function transformCallback(
// TODO: Make this not optional in v3
callback, once = false) {
    return window.__TAURI_INTERNALS__.transformCallback(callback, once);
}
class Channel {
    constructor(onmessage) {
        _Channel_onmessage.set(this, void 0);
        // the index is used as a mechanism to preserve message order
        _Channel_nextMessageIndex.set(this, 0);
        _Channel_pendingMessages.set(this, []);
        _Channel_messageEndIndex.set(this, void 0);
        __classPrivateFieldSet(this, _Channel_onmessage, onmessage || (() => { }), "f");
        this.id = transformCallback((rawMessage) => {
            const index = rawMessage.index;
            if ('end' in rawMessage) {
                if (index == __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")) {
                    this.cleanupCallback();
                }
                else {
                    __classPrivateFieldSet(this, _Channel_messageEndIndex, index, "f");
                }
                return;
            }
            const message = rawMessage.message;
            // Process the message if we're at the right order
            if (index == __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")) {
                __classPrivateFieldGet(this, _Channel_onmessage, "f").call(this, message);
                __classPrivateFieldSet(this, _Channel_nextMessageIndex, __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") + 1, "f");
                // process pending messages
                while (__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") in __classPrivateFieldGet(this, _Channel_pendingMessages, "f")) {
                    const message = __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")];
                    __classPrivateFieldGet(this, _Channel_onmessage, "f").call(this, message);
                    // eslint-disable-next-line @typescript-eslint/no-array-delete
                    delete __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")];
                    __classPrivateFieldSet(this, _Channel_nextMessageIndex, __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") + 1, "f");
                }
                if (__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") === __classPrivateFieldGet(this, _Channel_messageEndIndex, "f")) {
                    this.cleanupCallback();
                }
            }
            // Queue the message if we're not
            else {
                // eslint-disable-next-line security/detect-object-injection
                __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[index] = message;
            }
        });
    }
    cleanupCallback() {
        window.__TAURI_INTERNALS__.unregisterCallback(this.id);
    }
    set onmessage(handler) {
        __classPrivateFieldSet(this, _Channel_onmessage, handler, "f");
    }
    get onmessage() {
        return __classPrivateFieldGet(this, _Channel_onmessage, "f");
    }
    [(_Channel_onmessage = new WeakMap(), _Channel_nextMessageIndex = new WeakMap(), _Channel_pendingMessages = new WeakMap(), _Channel_messageEndIndex = new WeakMap(), SERIALIZE_TO_IPC_FN)]() {
        return `__CHANNEL__:${this.id}`;
    }
    toJSON() {
        // eslint-disable-next-line security/detect-object-injection
        return this[SERIALIZE_TO_IPC_FN]();
    }
}
class PluginListener {
    constructor(plugin, event, channelId) {
        this.plugin = plugin;
        this.event = event;
        this.channelId = channelId;
    }
    async unregister() {
        return invoke(`plugin:${this.plugin}|remove_listener`, {
            event: this.event,
            channelId: this.channelId
        });
    }
}
/**
 * Adds a listener to a plugin event.
 *
 * @returns The listener object to stop listening to the events.
 *
 * @since 2.0.0
 */
async function addPluginListener(plugin, event, cb) {
    const handler = new Channel(cb);
    try {
        await invoke(`plugin:${plugin}|register_listener`, {
            event,
            handler
        });
        return new PluginListener(plugin, event, handler.id);
    }
    catch {
        // TODO(v3): remove this fallback
        // note: we must try with camelCase here for backwards compatibility
        await invoke(`plugin:${plugin}|registerListener`, { event, handler });
        return new PluginListener(plugin, event, handler.id);
    }
}
/**
 * Get permission state for a plugin.
 *
 * This should be used by plugin authors to wrap their actual implementation.
 */
async function checkPermissions(plugin) {
    return invoke(`plugin:${plugin}|check_permissions`);
}
/**
 * Request permissions.
 *
 * This should be used by plugin authors to wrap their actual implementation.
 */
async function requestPermissions(plugin) {
    return invoke(`plugin:${plugin}|request_permissions`);
}
/**
 * Sends a message to the backend.
 * @example
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * await invoke('login', { user: 'tauri', password: 'poiwe3h4r5ip3yrhtew9ty' });
 * ```
 *
 * @param cmd The command name.
 * @param args The optional arguments to pass to the command.
 * @param options The request options.
 * @return A promise resolving or rejecting to the backend response.
 *
 * @since 1.0.0
 */
async function invoke(cmd, args = {}, options) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args, options);
}
/**
 * Convert a device file path to an URL that can be loaded by the webview.
 * Note that `asset:` and `http://asset.localhost` must be added to [`app.security.csp`](https://v2.tauri.app/reference/config/#csp-1) in `tauri.conf.json`.
 * Example CSP value: `"csp": "default-src 'self' ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost"` to use the asset protocol on image sources.
 *
 * Additionally, `"enable" : "true"` must be added to [`app.security.assetProtocol`](https://v2.tauri.app/reference/config/#assetprotocolconfig)
 * in `tauri.conf.json` and its access scope must be defined on the `scope` array on the same `assetProtocol` object.
 *
 * @param  filePath The file path.
 * @param  protocol The protocol to use. Defaults to `asset`. You only need to set this when using a custom protocol.
 * @example
 * ```typescript
 * import { appDataDir, join } from '@tauri-apps/api/path';
 * import { convertFileSrc } from '@tauri-apps/api/core';
 * const appDataDirPath = await appDataDir();
 * const filePath = await join(appDataDirPath, 'assets/video.mp4');
 * const assetUrl = convertFileSrc(filePath);
 *
 * const video = document.getElementById('my-video');
 * const source = document.createElement('source');
 * source.type = 'video/mp4';
 * source.src = assetUrl;
 * video.appendChild(source);
 * video.load();
 * ```
 *
 * @return the URL that can be used as source on the webview.
 *
 * @since 1.0.0
 */
function convertFileSrc(filePath, protocol = 'asset') {
    return window.__TAURI_INTERNALS__.convertFileSrc(filePath, protocol);
}
/**
 * A rust-backed resource stored through `tauri::Manager::resources_table` API.
 *
 * The resource lives in the main process and does not exist
 * in the Javascript world, and thus will not be cleaned up automatically
 * except on application exit. If you want to clean it up early, call {@linkcode Resource.close}
 *
 * @example
 * ```typescript
 * import { Resource, invoke } from '@tauri-apps/api/core';
 * export class DatabaseHandle extends Resource {
 *   static async open(path: string): Promise<DatabaseHandle> {
 *     const rid: number = await invoke('open_db', { path });
 *     return new DatabaseHandle(rid);
 *   }
 *
 *   async execute(sql: string): Promise<void> {
 *     await invoke('execute_sql', { rid: this.rid, sql });
 *   }
 * }
 * ```
 */
class Resource {
    get rid() {
        return __classPrivateFieldGet(this, _Resource_rid, "f");
    }
    constructor(rid) {
        _Resource_rid.set(this, void 0);
        __classPrivateFieldSet(this, _Resource_rid, rid, "f");
    }
    /**
     * Destroys and cleans up this resource from memory.
     * **You should not call any method on this object anymore and should drop any reference to it.**
     */
    async close() {
        return invoke('plugin:resources|close', {
            rid: this.rid
        });
    }
}
_Resource_rid = new WeakMap();
function isTauri() {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
    return !!(globalThis || window).isTauri;
}

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
/**
 * Access the system shell.
 * Allows you to spawn child processes and manage files and URLs using their default application.
 *
 * ## Security
 *
 * This API has a scope configuration that forces you to restrict the programs and arguments that can be used.
 *
 * ### Restricting access to the {@link open | `open`} API
 *
 * On the configuration object, `open: true` means that the {@link open} API can be used with any URL,
 * as the argument is validated with the `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+` regex.
 * You can change that regex by changing the boolean value to a string, e.g. `open: ^https://github.com/`.
 *
 * ### Restricting access to the {@link Command | `Command`} APIs
 *
 * The plugin permissions object has a `scope` field that defines an array of CLIs that can be used.
 * Each CLI is a configuration object `{ name: string, cmd: string, sidecar?: bool, args?: boolean | Arg[] }`.
 *
 * - `name`: the unique identifier of the command, passed to the {@link Command.create | Command.create function}.
 * If it's a sidecar, this must be the value defined on `tauri.conf.json > bundle > externalBin`.
 * - `cmd`: the program that is executed on this configuration. If it's a sidecar, this value is ignored.
 * - `sidecar`: whether the object configures a sidecar or a system program.
 * - `args`: the arguments that can be passed to the program. By default no arguments are allowed.
 *   - `true` means that any argument list is allowed.
 *   - `false` means that no arguments are allowed.
 *   - otherwise an array can be configured. Each item is either a string representing the fixed argument value
 *     or a `{ validator: string }` that defines a regex validating the argument value.
 *
 * #### Example scope configuration
 *
 * CLI: `git commit -m "the commit message"`
 *
 * Capability:
 * ```json
 * {
 *   "permissions": [
 *     {
 *       "identifier": "shell:allow-execute",
 *       "allow": [
 *         {
 *           "name": "run-git-commit",
 *           "cmd": "git",
 *           "args": ["commit", "-m", { "validator": "\\S+" }]
 *         }
 *       ]
 *     }
 *   ]
 * }
 * ```
 * Usage:
 * ```typescript
 * import { Command } from '@tauri-apps/plugin-shell'
 * Command.create('run-git-commit', ['commit', '-m', 'the commit message'])
 * ```
 *
 * Trying to execute any API with a program not configured on the scope results in a promise rejection due to denied access.
 *
 * @module
 */
/**
 * @since 2.0.0
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
class EventEmitter {
    constructor() {
        /** @ignore */
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
        this.eventListeners = Object.create(null);
    }
    /**
     * Alias for `emitter.on(eventName, listener)`.
     *
     * @since 2.0.0
     */
    addListener(eventName, listener) {
        return this.on(eventName, listener);
    }
    /**
     * Alias for `emitter.off(eventName, listener)`.
     *
     * @since 2.0.0
     */
    removeListener(eventName, listener) {
        return this.off(eventName, listener);
    }
    /**
     * Adds the `listener` function to the end of the listeners array for the
     * event named `eventName`. No checks are made to see if the `listener` has
     * already been added. Multiple calls passing the same combination of `eventName`and `listener` will result in the `listener` being added, and called, multiple
     * times.
     *
     * Returns a reference to the `EventEmitter`, so that calls can be chained.
     *
     * @since 2.0.0
     */
    on(eventName, listener) {
        if (eventName in this.eventListeners) {
            // eslint-disable-next-line security/detect-object-injection
            this.eventListeners[eventName].push(listener);
        }
        else {
            // eslint-disable-next-line security/detect-object-injection
            this.eventListeners[eventName] = [listener];
        }
        return this;
    }
    /**
     * Adds a **one-time**`listener` function for the event named `eventName`. The
     * next time `eventName` is triggered, this listener is removed and then invoked.
     *
     * Returns a reference to the `EventEmitter`, so that calls can be chained.
     *
     * @since 2.0.0
     */
    once(eventName, listener) {
        const wrapper = (arg) => {
            this.removeListener(eventName, wrapper);
            listener(arg);
        };
        return this.addListener(eventName, wrapper);
    }
    /**
     * Removes the all specified listener from the listener array for the event eventName
     * Returns a reference to the `EventEmitter`, so that calls can be chained.
     *
     * @since 2.0.0
     */
    off(eventName, listener) {
        if (eventName in this.eventListeners) {
            // eslint-disable-next-line security/detect-object-injection
            this.eventListeners[eventName] = this.eventListeners[eventName].filter((l) => l !== listener);
        }
        return this;
    }
    /**
     * Removes all listeners, or those of the specified eventName.
     *
     * Returns a reference to the `EventEmitter`, so that calls can be chained.
     *
     * @since 2.0.0
     */
    removeAllListeners(event) {
        if (event) {
            // eslint-disable-next-line security/detect-object-injection
            delete this.eventListeners[event];
        }
        else {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            this.eventListeners = Object.create(null);
        }
        return this;
    }
    /**
     * @ignore
     * Synchronously calls each of the listeners registered for the event named`eventName`, in the order they were registered, passing the supplied arguments
     * to each.
     *
     * @returns `true` if the event had listeners, `false` otherwise.
     *
     * @since 2.0.0
     */
    emit(eventName, arg) {
        if (eventName in this.eventListeners) {
            // eslint-disable-next-line security/detect-object-injection
            const listeners = this.eventListeners[eventName];
            for (const listener of listeners)
                listener(arg);
            return true;
        }
        return false;
    }
    /**
     * Returns the number of listeners listening to the event named `eventName`.
     *
     * @since 2.0.0
     */
    listenerCount(eventName) {
        if (eventName in this.eventListeners)
            // eslint-disable-next-line security/detect-object-injection
            return this.eventListeners[eventName].length;
        return 0;
    }
    /**
     * Adds the `listener` function to the _beginning_ of the listeners array for the
     * event named `eventName`. No checks are made to see if the `listener` has
     * already been added. Multiple calls passing the same combination of `eventName`and `listener` will result in the `listener` being added, and called, multiple
     * times.
     *
     * Returns a reference to the `EventEmitter`, so that calls can be chained.
     *
     * @since 2.0.0
     */
    prependListener(eventName, listener) {
        if (eventName in this.eventListeners) {
            // eslint-disable-next-line security/detect-object-injection
            this.eventListeners[eventName].unshift(listener);
        }
        else {
            // eslint-disable-next-line security/detect-object-injection
            this.eventListeners[eventName] = [listener];
        }
        return this;
    }
    /**
     * Adds a **one-time**`listener` function for the event named `eventName` to the_beginning_ of the listeners array. The next time `eventName` is triggered, this
     * listener is removed, and then invoked.
     *
     * Returns a reference to the `EventEmitter`, so that calls can be chained.
     *
     * @since 2.0.0
     */
    prependOnceListener(eventName, listener) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const wrapper = (arg) => {
            this.removeListener(eventName, wrapper);
            // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
            listener(arg);
        };
        return this.prependListener(eventName, wrapper);
    }
}
/**
 * @since 2.0.0
 */
class Child {
    constructor(pid) {
        this.pid = pid;
    }
    /**
     * Writes `data` to the `stdin`.
     *
     * @param data The message to write, either a string or a byte array.
     * @example
     * ```typescript
     * import { Command } from '@tauri-apps/plugin-shell';
     * const command = Command.create('node');
     * const child = await command.spawn();
     * await child.write('message');
     * await child.write([0, 1, 2, 3, 4, 5]);
     * ```
     *
     * @returns A promise indicating the success or failure of the operation.
     *
     * @since 2.0.0
     */
    async write(data) {
        await invoke('plugin:shell|stdin_write', {
            pid: this.pid,
            buffer: data
        });
    }
    /**
     * Kills the child process.
     *
     * @returns A promise indicating the success or failure of the operation.
     *
     * @since 2.0.0
     */
    async kill() {
        await invoke('plugin:shell|kill', {
            cmd: 'killChild',
            pid: this.pid
        });
    }
}
/**
 * The entry point for spawning child processes.
 * It emits the `close` and `error` events.
 * @example
 * ```typescript
 * import { Command } from '@tauri-apps/plugin-shell';
 * const command = Command.create('node');
 * command.on('close', data => {
 *   console.log(`command finished with code ${data.code} and signal ${data.signal}`)
 * });
 * command.on('error', error => console.error(`command error: "${error}"`));
 * command.stdout.on('data', line => console.log(`command stdout: "${line}"`));
 * command.stderr.on('data', line => console.log(`command stderr: "${line}"`));
 *
 * const child = await command.spawn();
 * console.log('pid:', child.pid);
 * ```
 *
 * @since 2.0.0
 *
 */
class Command extends EventEmitter {
    /**
     * @ignore
     * Creates a new `Command` instance.
     *
     * @param program The program name to execute.
     * It must be configured in your project's capabilities.
     * @param args Program arguments.
     * @param options Spawn options.
     */
    constructor(program, args = [], options) {
        super();
        /** Event emitter for the `stdout`. Emits the `data` event. */
        this.stdout = new EventEmitter();
        /** Event emitter for the `stderr`. Emits the `data` event. */
        this.stderr = new EventEmitter();
        this.program = program;
        this.args = typeof args === 'string' ? [args] : args;
        this.options = options ?? {};
    }
    /**
     * Creates a command to execute the given program.
     * @example
     * ```typescript
     * import { Command } from '@tauri-apps/plugin-shell';
     * const command = Command.create('my-app', ['run', 'tauri']);
     * const output = await command.execute();
     * ```
     *
     * @param program The program to execute.
     * It must be configured in your project's capabilities.
     */
    static create(program, args = [], options) {
        return new Command(program, args, options);
    }
    /**
     * Creates a command to execute the given sidecar program.
     * @example
     * ```typescript
     * import { Command } from '@tauri-apps/plugin-shell';
     * const command = Command.sidecar('my-sidecar');
     * const output = await command.execute();
     * ```
     *
     * @param program The program to execute.
     * It must be configured in your project's capabilities.
     */
    static sidecar(program, args = [], options) {
        const instance = new Command(program, args, options);
        instance.options.sidecar = true;
        return instance;
    }
    /**
     * Executes the command as a child process, returning a handle to it.
     *
     * @returns A promise resolving to the child process handle.
     *
     * @since 2.0.0
     */
    async spawn() {
        const program = this.program;
        const args = this.args;
        const options = this.options;
        if (typeof args === 'object') {
            Object.freeze(args);
        }
        const onEvent = new Channel();
        onEvent.onmessage = (event) => {
            switch (event.event) {
                case 'Error':
                    this.emit('error', event.payload);
                    break;
                case 'Terminated':
                    this.emit('close', event.payload);
                    break;
                case 'Stdout':
                    this.stdout.emit('data', event.payload);
                    break;
                case 'Stderr':
                    this.stderr.emit('data', event.payload);
                    break;
            }
        };
        return await invoke('plugin:shell|spawn', {
            program,
            args,
            options,
            onEvent
        }).then((pid) => new Child(pid));
    }
    /**
     * Executes the command as a child process, waiting for it to finish and collecting all of its output.
     * @example
     * ```typescript
     * import { Command } from '@tauri-apps/plugin-shell';
     * const output = await Command.create('echo', 'message').execute();
     * assert(output.code === 0);
     * assert(output.signal === null);
     * assert(output.stdout === 'message');
     * assert(output.stderr === '');
     * ```
     *
     * @returns A promise resolving to the child process output.
     *
     * @since 2.0.0
     */
    async execute() {
        const program = this.program;
        const args = this.args;
        const options = this.options;
        if (typeof args === 'object') {
            Object.freeze(args);
        }
        return await invoke('plugin:shell|execute', {
            program,
            args,
            options
        });
    }
}
/**
 * Opens a path or URL with the system's default app,
 * or the one specified with `openWith`.
 *
 * The `openWith` value must be one of `firefox`, `google chrome`, `chromium` `safari`,
 * `open`, `start`, `xdg-open`, `gio`, `gnome-open`, `kde-open` or `wslview`.
 *
 * @example
 * ```typescript
 * import { open } from '@tauri-apps/plugin-shell';
 * // opens the given URL on the default browser:
 * await open('https://github.com/tauri-apps/tauri');
 * // opens the given URL using `firefox`:
 * await open('https://github.com/tauri-apps/tauri', 'firefox');
 * // opens a file using the default program:
 * await open('/path/to/file');
 * ```
 *
 * @param path The path or URL to open.
 * This value is matched against the string regex defined on `tauri.conf.json > plugins > shell > open`,
 * which defaults to `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`.
 * @param openWith The app to open the file or URL with.
 * Defaults to the system default application for the specified path type.
 *
 * @since 2.0.0
 */
async function open(path, openWith) {
    await invoke('plugin:shell|open', {
        path,
        with: openWith
    });
}

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
/**
 * The event system allows you to emit events to the backend and listen to events from it.
 *
 * This package is also accessible with `window.__TAURI__.event` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */
/**
 * @since 1.1.0
 */
var TauriEvent;
(function (TauriEvent) {
    TauriEvent["WINDOW_RESIZED"] = "tauri://resize";
    TauriEvent["WINDOW_MOVED"] = "tauri://move";
    TauriEvent["WINDOW_CLOSE_REQUESTED"] = "tauri://close-requested";
    TauriEvent["WINDOW_DESTROYED"] = "tauri://destroyed";
    TauriEvent["WINDOW_FOCUS"] = "tauri://focus";
    TauriEvent["WINDOW_BLUR"] = "tauri://blur";
    TauriEvent["WINDOW_SCALE_FACTOR_CHANGED"] = "tauri://scale-change";
    TauriEvent["WINDOW_THEME_CHANGED"] = "tauri://theme-changed";
    TauriEvent["WINDOW_CREATED"] = "tauri://window-created";
    TauriEvent["WEBVIEW_CREATED"] = "tauri://webview-created";
    TauriEvent["DRAG_ENTER"] = "tauri://drag-enter";
    TauriEvent["DRAG_OVER"] = "tauri://drag-over";
    TauriEvent["DRAG_DROP"] = "tauri://drag-drop";
    TauriEvent["DRAG_LEAVE"] = "tauri://drag-leave";
})(TauriEvent || (TauriEvent = {}));
/**
 * Unregister the event listener associated with the given name and id.
 *
 * @ignore
 * @param event The event name
 * @param eventId Event identifier
 * @returns
 */
async function _unlisten(event, eventId) {
    window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(event, eventId);
    await invoke('plugin:event|unlisten', {
        event,
        eventId
    });
}
/**
 * Listen to an emitted event to any {@link EventTarget|target}.
 *
 * @example
 * ```typescript
 * import { listen } from '@tauri-apps/api/event';
 * const unlisten = await listen<string>('error', (event) => {
 *   console.log(`Got error, payload: ${event.payload}`);
 * });
 *
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @param options Event listening options.
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
 *
 * @since 1.0.0
 */
async function listen(event, handler, options) {
    var _a;
    const target = typeof (options === null || options === void 0 ? void 0 : options.target) === 'string'
        ? { kind: 'AnyLabel', label: options.target }
        : ((_a = options === null || options === void 0 ? void 0 : options.target) !== null && _a !== void 0 ? _a : { kind: 'Any' });
    return invoke('plugin:event|listen', {
        event,
        target,
        handler: transformCallback(handler)
    }).then((eventId) => {
        return async () => _unlisten(event, eventId);
    });
}
/**
 * Listens once to an emitted event to any {@link EventTarget|target}.
 *
 * @example
 * ```typescript
 * import { once } from '@tauri-apps/api/event';
 * interface LoadedPayload {
 *   loggedIn: boolean,
 *   token: string
 * }
 * const unlisten = await once<LoadedPayload>('loaded', (event) => {
 *   console.log(`App is loaded, loggedIn: ${event.payload.loggedIn}, token: ${event.payload.token}`);
 * });
 *
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @param options Event listening options.
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
 *
 * @since 1.0.0
 */
async function once(event, handler, options) {
    return listen(event, (eventData) => {
        void _unlisten(event, eventData.id);
        handler(eventData);
    }, options);
}
/**
 * Emits an event to all {@link EventTarget|targets}.
 *
 * @example
 * ```typescript
 * import { emit } from '@tauri-apps/api/event';
 * await emit('frontend-loaded', { loggedIn: true, token: 'authToken' });
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param payload Event payload.
 *
 * @since 1.0.0
 */
async function emit(event, payload) {
    await invoke('plugin:event|emit', {
        event,
        payload
    });
}
/**
 * Emits an event to all {@link EventTarget|targets} matching the given target.
 *
 * @example
 * ```typescript
 * import { emitTo } from '@tauri-apps/api/event';
 * await emitTo('main', 'frontend-loaded', { loggedIn: true, token: 'authToken' });
 * ```
 *
 * @param target Label of the target Window/Webview/WebviewWindow or raw {@link EventTarget} object.
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param payload Event payload.
 *
 * @since 2.0.0
 */
async function emitTo(target, event, payload) {
    const eventTarget = typeof target === 'string' ? { kind: 'AnyLabel', label: target } : target;
    await invoke('plugin:event|emit_to', {
        target: eventTarget,
        event,
        payload
    });
}

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
/**
 * Create a new Store or load the existing store with the path.
 *
 * @example
 * ```typescript
 * import { Store } from '@tauri-apps/api/store';
 * const store = await Store.load('store.json');
 * ```
 *
 * @param path Path to save the store in `app_data_dir`
 * @param options Store configuration options
 */
async function load(path, options) {
    return await Store.load(path, options);
}
/**
 * Gets an already loaded store.
 *
 * If the store is not loaded, returns `null`. In this case you must {@link Store.load load} it.
 *
 * This function is more useful when you already know the store is loaded
 * and just need to access its instance. Prefer {@link Store.load} otherwise.
 *
 * @example
 * ```typescript
 * import { getStore } from '@tauri-apps/api/store';
 * const store = await getStore('store.json');
 * ```
 *
 * @param path Path of the store.
 */
async function getStore(path) {
    return await Store.get(path);
}
/**
 * A lazy loaded key-value store persisted by the backend layer.
 */
class LazyStore {
    get store() {
        if (!this._store) {
            this._store = load(this.path, this.options);
        }
        return this._store;
    }
    /**
     * Note that the options are not applied if someone else already created the store
     * @param path Path to save the store in `app_data_dir`
     * @param options Store configuration options
     */
    constructor(path, options) {
        this.path = path;
        this.options = options;
    }
    /**
     * Init/load the store if it's not loaded already
     */
    async init() {
        await this.store;
    }
    async set(key, value) {
        return (await this.store).set(key, value);
    }
    async get(key) {
        return (await this.store).get(key);
    }
    async has(key) {
        return (await this.store).has(key);
    }
    async delete(key) {
        return (await this.store).delete(key);
    }
    async clear() {
        await (await this.store).clear();
    }
    async reset() {
        await (await this.store).reset();
    }
    async keys() {
        return (await this.store).keys();
    }
    async values() {
        return (await this.store).values();
    }
    async entries() {
        return (await this.store).entries();
    }
    async length() {
        return (await this.store).length();
    }
    async reload(options) {
        await (await this.store).reload(options);
    }
    async save() {
        await (await this.store).save();
    }
    async onKeyChange(key, cb) {
        return (await this.store).onKeyChange(key, cb);
    }
    async onChange(cb) {
        return (await this.store).onChange(cb);
    }
    async close() {
        if (this._store) {
            await (await this._store).close();
        }
    }
}
/**
 * A key-value store persisted by the backend layer.
 */
class Store extends Resource {
    constructor(rid) {
        super(rid);
    }
    /**
     * Create a new Store or load the existing store with the path.
     *
     * @example
     * ```typescript
     * import { Store } from '@tauri-apps/api/store';
     * const store = await Store.load('store.json');
     * ```
     *
     * @param path Path to save the store in `app_data_dir`
     * @param options Store configuration options
     */
    static async load(path, options) {
        const rid = await invoke('plugin:store|load', {
            path,
            options
        });
        return new Store(rid);
    }
    /**
     * Gets an already loaded store.
     *
     * If the store is not loaded, returns `null`. In this case you must {@link Store.load load} it.
     *
     * This function is more useful when you already know the store is loaded
     * and just need to access its instance. Prefer {@link Store.load} otherwise.
     *
     * @example
     * ```typescript
     * import { Store } from '@tauri-apps/api/store';
     * let store = await Store.get('store.json');
     * if (!store) {
     *   store = await Store.load('store.json');
     * }
     * ```
     *
     * @param path Path of the store.
     */
    static async get(path) {
        return await invoke('plugin:store|get_store', { path }).then((rid) => (rid ? new Store(rid) : null));
    }
    async set(key, value) {
        await invoke('plugin:store|set', {
            rid: this.rid,
            key,
            value
        });
    }
    async get(key) {
        const [value, exists] = await invoke('plugin:store|get', {
            rid: this.rid,
            key
        });
        return exists ? value : undefined;
    }
    async has(key) {
        return await invoke('plugin:store|has', {
            rid: this.rid,
            key
        });
    }
    async delete(key) {
        return await invoke('plugin:store|delete', {
            rid: this.rid,
            key
        });
    }
    async clear() {
        await invoke('plugin:store|clear', { rid: this.rid });
    }
    async reset() {
        await invoke('plugin:store|reset', { rid: this.rid });
    }
    async keys() {
        return await invoke('plugin:store|keys', { rid: this.rid });
    }
    async values() {
        return await invoke('plugin:store|values', { rid: this.rid });
    }
    async entries() {
        return await invoke('plugin:store|entries', { rid: this.rid });
    }
    async length() {
        return await invoke('plugin:store|length', { rid: this.rid });
    }
    async reload(options) {
        await invoke('plugin:store|reload', { rid: this.rid, ...options });
    }
    async save() {
        await invoke('plugin:store|save', { rid: this.rid });
    }
    async onKeyChange(key, cb) {
        return await listen('store://change', (event) => {
            if (event.payload.resourceId === this.rid && event.payload.key === key) {
                cb(event.payload.exists ? event.payload.value : undefined);
            }
        });
    }
    async onChange(cb) {
        return await listen('store://change', (event) => {
            if (event.payload.resourceId === this.rid) {
                cb(event.payload.key, event.payload.exists ? event.payload.value : undefined);
            }
        });
    }
}

const token = ref(null);
const user = ref(null);
const repo = ref("");
let pollInterval = null;
let pollTimeout = null;
function clearPolling() {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
  if (pollTimeout) {
    clearTimeout(pollTimeout);
    pollTimeout = null;
  }
}
function useGitHubAuth() {
  const loading = ref(false);
  const userCode = ref("");
  const error = ref(null);
  onUnmounted(clearPolling);
  async function init() {
    try {
      const store = await load("settings.json");
      token.value = await store.get("token") || null;
      repo.value = await store.get("repo") || "";
      if (token.value) {
        user.value = await invoke("get_user", { token: token.value });
      }
    } catch {
      token.value = null;
      user.value = null;
      const store = await load("settings.json");
      await store.delete("token");
      await store.save();
    }
  }
  async function startLogin() {
    if (loading.value) return;
    clearPolling();
    loading.value = true;
    error.value = null;
    try {
      const res = await invoke("start_oauth");
      userCode.value = res.user_code;
      await open(res.verification_uri);
      pollTimeout = setTimeout(() => {
        clearPolling();
        loading.value = false;
        userCode.value = "";
        error.value = "Authentication expired. Please try again.";
      }, res.expires_in * 1e3);
      pollInterval = setInterval(async () => {
        try {
          const t = await invoke("poll_oauth", { deviceCode: res.device_code });
          if (t) {
            clearPolling();
            token.value = t;
            user.value = await invoke("get_user", { token: t });
            const store = await load("settings.json");
            await store.set("token", t);
            await store.save();
            loading.value = false;
            userCode.value = "";
          }
        } catch (e) {
          clearPolling();
          loading.value = false;
          userCode.value = "";
          error.value = String(e);
        }
      }, res.interval * 1e3);
    } catch (e) {
      loading.value = false;
      error.value = String(e);
    }
  }
  async function logout() {
    clearPolling();
    token.value = null;
    user.value = null;
    const store = await load("settings.json");
    await store.delete("token");
    await store.save();
  }
  async function setRepo(r) {
    repo.value = r;
    const store = await load("settings.json");
    await store.set("repo", r);
    await store.save();
  }
  return { token, user, repo, loading, userCode, error, init, startLogin, logout, setRepo };
}

const accent = ref("cyan");
const colors = {
  pink: "#ff2d6a",
  cyan: "#00f0ff",
  purple: "#b026ff",
  green: "#39ff14",
  orange: "#ff6b35",
  yellow: "#ffd700"
};
function useAccentColor() {
  const accentHex = computed(() => colors[accent.value]);
  const accentClass = computed(() => `text-cyber-${accent.value}`);
  const bgClass = computed(() => `bg-cyber-${accent.value}`);
  const borderClass = computed(() => `border-cyber-${accent.value}`);
  async function init() {
    try {
      const store = await load("settings.json");
      const saved = await store.get("accent");
      if (saved && colors[saved]) {
        accent.value = saved;
        document.documentElement.style.setProperty("--accent", colors[saved]);
      }
    } catch {
    }
  }
  async function setAccent(color) {
    accent.value = color;
    document.documentElement.style.setProperty("--accent", colors[color]);
    try {
      const store = await load("settings.json");
      await store.set("accent", color);
      await store.save();
    } catch {
    }
  }
  return { accent, accentHex, accentClass, bgClass, borderClass, init, setAccent, colors };
}

const _sfc_main = /* @__PURE__ */ defineComponent({
  __name: "AuthButton",
  setup(__props, { expose: __expose }) {
    __expose();
    const { user, loading, userCode, error, startLogin, logout } = useGitHubAuth();
    const { accentHex } = useAccentColor();
    const __returned__ = { user, loading, userCode, error, startLogin, logout, accentHex };
    Object.defineProperty(__returned__, "__isScriptSetup", { enumerable: false, value: true });
    return __returned__;
  }
});

const _hoisted_1 = {
  key: 0,
  class: "flex items-center gap-4 animate-fade-in pl-2 pr-1 py-1 bg-white/5 border border-white/10 rounded-full hover:bg-white/10 transition-colors"
};
const _hoisted_2 = { class: "flex items-center gap-3" };
const _hoisted_3 = ["src", "alt"];
const _hoisted_4 = { class: "text-sm font-medium text-gray-200 hidden sm:block" };
const _hoisted_5 = {
  key: 1,
  class: "glass-panel px-6 py-4 rounded-xl text-center animate-fade-in"
};
const _hoisted_6 = {
  key: 2,
  class: "flex flex-col items-end"
};
const _hoisted_7 = ["disabled"];
const _hoisted_8 = { class: "relative flex items-center gap-2" };
const _hoisted_9 = {
  key: 0,
  class: "animate-spin h-4 w-4 text-black",
  xmlns: "http://www.w3.org/2000/svg",
  fill: "none",
  viewBox: "0 0 24 24"
};
const _hoisted_10 = {
  key: 1,
  class: "w-4 h-4",
  fill: "currentColor",
  viewBox: "0 0 24 24"
};
const _hoisted_11 = {
  key: 0,
  class: "text-red-400 text-[10px] mt-2 font-medium bg-red-500/10 px-2 py-1 rounded"
};
function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  return openBlock(), createElementBlock("div", null, [
    createVNode(Transition, {
      name: "fade",
      mode: "out-in"
    }, {
      default: withCtx(() => [
        $setup.user ? (openBlock(), createElementBlock("div", _hoisted_1, [
          createBaseVNode("div", _hoisted_2, [
            createBaseVNode("img", {
              src: $setup.user.avatar_url,
              alt: $setup.user.login,
              class: "w-6 h-6 rounded-full ring-2 ring-transparent",
              style: normalizeStyle({ borderColor: $setup.accentHex })
            }, null, 12, _hoisted_3),
            createBaseVNode(
              "span",
              _hoisted_4,
              toDisplayString($setup.user.login),
              1
              /* TEXT */
            )
          ]),
          createBaseVNode("button", {
            onClick: _cache[0] || (_cache[0] = (...args) => $setup.logout && $setup.logout(...args)),
            class: "w-7 h-7 flex items-center justify-center rounded-full bg-white/5 text-gray-400 hover:text-white hover:bg-red-500/80 hover:scale-105 transition-all",
            title: "Logout"
          }, [..._cache[2] || (_cache[2] = [
            createBaseVNode(
              "svg",
              {
                class: "w-3.5 h-3.5",
                fill: "none",
                viewBox: "0 0 24 24",
                stroke: "currentColor"
              },
              [
                createBaseVNode("path", {
                  "stroke-linecap": "round",
                  "stroke-linejoin": "round",
                  "stroke-width": "2",
                  d: "M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                })
              ],
              -1
              /* CACHED */
            )
          ])])
        ])) : $setup.userCode ? (openBlock(), createElementBlock("div", _hoisted_5, [
          _cache[3] || (_cache[3] = createBaseVNode(
            "p",
            { class: "text-xs text-gray-500 uppercase tracking-widest mb-2 font-medium" },
            "Device Code",
            -1
            /* CACHED */
          )),
          createBaseVNode(
            "div",
            {
              class: "text-2xl font-mono font-bold tracking-[0.2em] text-glow select-all",
              style: normalizeStyle({ color: $setup.accentHex })
            },
            toDisplayString($setup.userCode),
            5
            /* TEXT, STYLE */
          ),
          _cache[4] || (_cache[4] = createBaseVNode(
            "p",
            { class: "text-[10px] text-gray-600 mt-2" },
            "Enter this code on GitHub",
            -1
            /* CACHED */
          ))
        ])) : (openBlock(), createElementBlock("div", _hoisted_6, [
          createBaseVNode("button", {
            onClick: _cache[1] || (_cache[1] = (...args) => $setup.startLogin && $setup.startLogin(...args)),
            disabled: $setup.loading,
            class: "group relative px-6 py-2.5 bg-white text-black font-semibold rounded-full text-sm transition-all hover:scale-105 hover:shadow-[0_0_20px_-5px_rgba(255,255,255,0.3)] disabled:opacity-50 disabled:hover:scale-100 disabled:cursor-not-allowed"
          }, [
            createBaseVNode("span", _hoisted_8, [
              $setup.loading ? (openBlock(), createElementBlock("svg", _hoisted_9, [..._cache[5] || (_cache[5] = [
                createBaseVNode(
                  "circle",
                  {
                    class: "opacity-25",
                    cx: "12",
                    cy: "12",
                    r: "10",
                    stroke: "currentColor",
                    "stroke-width": "4"
                  },
                  null,
                  -1
                  /* CACHED */
                ),
                createBaseVNode(
                  "path",
                  {
                    class: "opacity-75",
                    fill: "currentColor",
                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  },
                  null,
                  -1
                  /* CACHED */
                )
              ])])) : (openBlock(), createElementBlock("svg", _hoisted_10, [..._cache[6] || (_cache[6] = [
                createBaseVNode(
                  "path",
                  {
                    "fill-rule": "evenodd",
                    d: "M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0122 12.017C22 6.484 17.522 2 12 2z",
                    "clip-rule": "evenodd"
                  },
                  null,
                  -1
                  /* CACHED */
                )
              ])])),
              createTextVNode(
                " " + toDisplayString($setup.loading ? "Connecting..." : "Sign In with GitHub"),
                1
                /* TEXT */
              )
            ])
          ], 8, _hoisted_7),
          $setup.error ? (openBlock(), createElementBlock(
            "p",
            _hoisted_11,
            toDisplayString($setup.error),
            1
            /* TEXT */
          )) : createCommentVNode("v-if", true)
        ]))
      ]),
      _: 1
      /* STABLE */
    })
  ]);
}
const AuthButton = /* @__PURE__ */ _export_sfc(_sfc_main, [["render", _sfc_render], ["__file", "AuthButton.vue"]]);

export { AuthButton as default };
//# sourceMappingURL=AuthButton-BevI0nZC.js.map
