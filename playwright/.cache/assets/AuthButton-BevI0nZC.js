/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 20 modules
 */

import { r as ref, o as onUnmounted, c as computed, d as defineComponent, a as createElementBlock, b as openBlock, e as createVNode, T as Transition, w as withCtx, f as createBaseVNode, n as normalizeStyle, t as toDisplayString, g as createCommentVNode, h as createTextVNode } from './index-CkkE85_b.js';
import { _ as _export_sfc } from './_plugin-vue_export-helper-pcqpp-6-.js';

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

var _Channel_onmessage, _Channel_nextMessageIndex, _Channel_pendingMessages, _Channel_messageEndIndex, _Resource_rid;

const SERIALIZE_TO_IPC_FN = '__TAURI_TO_IPC_KEY__';

function transformCallback(

callback, once = false) {
    return window.__TAURI_INTERNALS__.transformCallback(callback, once);
}
class Channel {
    constructor(onmessage) {
        _Channel_onmessage.set(this, void 0);
        
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
            
            if (index == __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")) {
                __classPrivateFieldGet(this, _Channel_onmessage, "f").call(this, message);
                __classPrivateFieldSet(this, _Channel_nextMessageIndex, __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") + 1, "f");
                
                while (__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") in __classPrivateFieldGet(this, _Channel_pendingMessages, "f")) {
                    const message = __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")];
                    __classPrivateFieldGet(this, _Channel_onmessage, "f").call(this, message);
                    
                    delete __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")];
                    __classPrivateFieldSet(this, _Channel_nextMessageIndex, __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") + 1, "f");
                }
                if (__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") === __classPrivateFieldGet(this, _Channel_messageEndIndex, "f")) {
                    this.cleanupCallback();
                }
            }
            
            else {
                
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

        await invoke(`plugin:${plugin}|registerListener`, { event, handler });
        return new PluginListener(plugin, event, handler.id);
    }
}

async function checkPermissions(plugin) {
    return invoke(`plugin:${plugin}|check_permissions`);
}

async function requestPermissions(plugin) {
    return invoke(`plugin:${plugin}|request_permissions`);
}

async function invoke(cmd, args = {}, options) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args, options);
}

function convertFileSrc(filePath, protocol = 'asset') {
    return window.__TAURI_INTERNALS__.convertFileSrc(filePath, protocol);
}

class Resource {
    get rid() {
        return __classPrivateFieldGet(this, _Resource_rid, "f");
    }
    constructor(rid) {
        _Resource_rid.set(this, void 0);
        __classPrivateFieldSet(this, _Resource_rid, rid, "f");
    }
    
    async close() {
        return invoke('plugin:resources|close', {
            rid: this.rid
        });
    }
}
_Resource_rid = new WeakMap();
function isTauri() {
    
    return !!(globalThis || window).isTauri;
}

class EventEmitter {
    constructor() {

        this.eventListeners = Object.create(null);
    }
    
    addListener(eventName, listener) {
        return this.on(eventName, listener);
    }
    
    removeListener(eventName, listener) {
        return this.off(eventName, listener);
    }
    
    on(eventName, listener) {
        if (eventName in this.eventListeners) {
            
            this.eventListeners[eventName].push(listener);
        }
        else {
            
            this.eventListeners[eventName] = [listener];
        }
        return this;
    }
    
    once(eventName, listener) {
        const wrapper = (arg) => {
            this.removeListener(eventName, wrapper);
            listener(arg);
        };
        return this.addListener(eventName, wrapper);
    }
    
    off(eventName, listener) {
        if (eventName in this.eventListeners) {
            
            this.eventListeners[eventName] = this.eventListeners[eventName].filter((l) => l !== listener);
        }
        return this;
    }
    
    removeAllListeners(event) {
        if (event) {
            
            delete this.eventListeners[event];
        }
        else {
            
            this.eventListeners = Object.create(null);
        }
        return this;
    }
    
    emit(eventName, arg) {
        if (eventName in this.eventListeners) {
            
            const listeners = this.eventListeners[eventName];
            for (const listener of listeners)
                listener(arg);
            return true;
        }
        return false;
    }
    
    listenerCount(eventName) {
        if (eventName in this.eventListeners)
            
            return this.eventListeners[eventName].length;
        return 0;
    }
    
    prependListener(eventName, listener) {
        if (eventName in this.eventListeners) {
            
            this.eventListeners[eventName].unshift(listener);
        }
        else {
            
            this.eventListeners[eventName] = [listener];
        }
        return this;
    }
    
    prependOnceListener(eventName, listener) {
        
        const wrapper = (arg) => {
            this.removeListener(eventName, wrapper);
            
            listener(arg);
        };
        return this.prependListener(eventName, wrapper);
    }
}

class Child {
    constructor(pid) {
        this.pid = pid;
    }
    
    async write(data) {
        await invoke('plugin:shell|stdin_write', {
            pid: this.pid,
            buffer: data
        });
    }
    
    async kill() {
        await invoke('plugin:shell|kill', {
            cmd: 'killChild',
            pid: this.pid
        });
    }
}

class Command extends EventEmitter {
    
    constructor(program, args = [], options) {
        super();
        
        this.stdout = new EventEmitter();
        
        this.stderr = new EventEmitter();
        this.program = program;
        this.args = typeof args === 'string' ? [args] : args;
        this.options = options ?? {};
    }
    
    static create(program, args = [], options) {
        return new Command(program, args, options);
    }
    
    static sidecar(program, args = [], options) {
        const instance = new Command(program, args, options);
        instance.options.sidecar = true;
        return instance;
    }
    
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

async function open(path, openWith) {
    await invoke('plugin:shell|open', {
        path,
        with: openWith
    });
}

var TauriEvent;
(function (TauriEvent) {
    TauriEvent["WINDOW_RESIZED"] = "tauri:
    TauriEvent["WINDOW_MOVED"] = "tauri:
    TauriEvent["WINDOW_CLOSE_REQUESTED"] = "tauri:
    TauriEvent["WINDOW_DESTROYED"] = "tauri:
    TauriEvent["WINDOW_FOCUS"] = "tauri:
    TauriEvent["WINDOW_BLUR"] = "tauri:
    TauriEvent["WINDOW_SCALE_FACTOR_CHANGED"] = "tauri:
    TauriEvent["WINDOW_THEME_CHANGED"] = "tauri:
    TauriEvent["WINDOW_CREATED"] = "tauri:
    TauriEvent["WEBVIEW_CREATED"] = "tauri:
    TauriEvent["DRAG_ENTER"] = "tauri:
    TauriEvent["DRAG_OVER"] = "tauri:
    TauriEvent["DRAG_DROP"] = "tauri:
    TauriEvent["DRAG_LEAVE"] = "tauri:
})(TauriEvent || (TauriEvent = {}));

async function _unlisten(event, eventId) {
    window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(event, eventId);
    await invoke('plugin:event|unlisten', {
        event,
        eventId
    });
}

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

async function once(event, handler, options) {
    return listen(event, (eventData) => {
        void _unlisten(event, eventData.id);
        handler(eventData);
    }, options);
}

async function emit(event, payload) {
    await invoke('plugin:event|emit', {
        event,
        payload
    });
}

async function emitTo(target, event, payload) {
    const eventTarget = typeof target === 'string' ? { kind: 'AnyLabel', label: target } : target;
    await invoke('plugin:event|emit_to', {
        target: eventTarget,
        event,
        payload
    });
}

async function load(path, options) {
    return await Store.load(path, options);
}

async function getStore(path) {
    return await Store.get(path);
}

class LazyStore {
    get store() {
        if (!this._store) {
            this._store = load(this.path, this.options);
        }
        return this._store;
    }
    
    constructor(path, options) {
        this.path = path;
        this.options = options;
    }
    
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

class Store extends Resource {
    constructor(rid) {
        super(rid);
    }
    
    static async load(path, options) {
        const rid = await invoke('plugin:store|load', {
            path,
            options
        });
        return new Store(rid);
    }
    
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
        return await listen('store:
            if (event.payload.resourceId === this.rid && event.payload.key === key) {
                cb(event.payload.exists ? event.payload.value : undefined);
            }
        });
    }
    async onChange(cb) {
        return await listen('store:
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

const _sfc_main =  defineComponent({
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
  xmlns: "http:
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
              
            )
          ])])
        ])) : $setup.userCode ? (openBlock(), createElementBlock("div", _hoisted_5, [
          _cache[3] || (_cache[3] = createBaseVNode(
            "p",
            { class: "text-xs text-gray-500 uppercase tracking-widest mb-2 font-medium" },
            "Device Code",
            -1
            
          )),
          createBaseVNode(
            "div",
            {
              class: "text-2xl font-mono font-bold tracking-[0.2em] text-glow select-all",
              style: normalizeStyle({ color: $setup.accentHex })
            },
            toDisplayString($setup.userCode),
            5
            
          ),
          _cache[4] || (_cache[4] = createBaseVNode(
            "p",
            { class: "text-[10px] text-gray-600 mt-2" },
            "Enter this code on GitHub",
            -1
            
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
                  
                )
              ])])),
              createTextVNode(
                " " + toDisplayString($setup.loading ? "Connecting..." : "Sign In with GitHub"),
                1
                
              )
            ])
          ], 8, _hoisted_7),
          $setup.error ? (openBlock(), createElementBlock(
            "p",
            _hoisted_11,
            toDisplayString($setup.error),
            1
            
          )) : createCommentVNode("v-if", true)
        ]))
      ]),
      _: 1
      
    })
  ]);
}
const AuthButton =  _export_sfc(_sfc_main, [["render", _sfc_render], ["__file", "AuthButton.vue"]]);

export { AuthButton as default };