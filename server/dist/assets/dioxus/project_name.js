import { create, update_memory, save_template, get_node, initilize } from './snippets/dioxus-interpreter-js-85a0a2f8acce2e86/inline0.js';
import { setAttributeInner } from './snippets/dioxus-interpreter-js-85a0a2f8acce2e86/src/common.js';
import { get_form_data } from './snippets/dioxus-web-a95d8cc6c91ff8eb/inline0.js';
import { Dioxus } from './snippets/dioxus-web-a95d8cc6c91ff8eb/src/eval.js';
import * as __wbg_star0 from './snippets/dioxus-interpreter-js-85a0a2f8acce2e86/inline0.js';

let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error('expected a boolean argument');
    }
}

let WASM_VECTOR_LEN = 0;

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (typeof(arg) !== 'string') throw new Error('expected a string argument');

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error('expected a number argument');
}

let cachedFloat64Memory0 = null;

function getFloat64Memory0() {
    if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
        cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64Memory0;
}

function _assertBigInt(n) {
    if (typeof(n) !== 'bigint') throw new Error('expected a bigint argument');
}

let cachedBigInt64Memory0 = null;

function getBigInt64Memory0() {
    if (cachedBigInt64Memory0 === null || cachedBigInt64Memory0.byteLength === 0) {
        cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer);
    }
    return cachedBigInt64Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
}
function __wbg_adapter_48(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke1_mut__h05487f7cb3c12108(arg0, arg1, addHeapObject(arg2));
}

function makeClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b);
                state.a = 0;

            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_51(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd2f12820f349c387(arg0, arg1);
}

function __wbg_adapter_54(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke1_mut__h95c71ccbb6666a76(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_57(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke0_mut__h6945be2091116baf(arg0, arg1);
}

let stack_pointer = 128;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}
function __wbg_adapter_60(arg0, arg1, arg2) {
    try {
        _assertNum(arg0);
        _assertNum(arg1);
        wasm.wasm_bindgen__convert__closures__invoke1_mut_ref__hfa5baafb966e7e14(arg0, arg1, addBorrowedObject(arg2));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

function __wbg_adapter_63(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke1_mut__h1e9b2f3a26745dc3(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_68(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke1_mut__hc87f0770b0e2424d(arg0, arg1, addHeapObject(arg2));
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(takeObject(slice[i]));
    }
    return result;
}

function getCachedStringFromWasm0(ptr, len) {
    if (ptr === 0) {
        return getObject(len);
    } else {
        return getStringFromWasm0(ptr, len);
    }
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_new_abd8a48ffd951779 = function() { return logError(function (arg0) {
        const ret = new Dioxus(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbg_rustSend_6175a678357f46b1 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).rustSend(takeObject(arg1));
    }, arguments) };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getformdata_7206b99ef44b788e = function() { return logError(function (arg0) {
        const ret = get_form_data(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_string = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'string';
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = getObject(arg0);
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        _assertNum(ret);
        return ret;
    };
    imports.wbg.__wbindgen_is_bigint = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'bigint';
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_i64 = function(arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = getObject(arg0) === getObject(arg1);
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
        const ret = BigInt.asUintN(64, arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'number' ? obj : undefined;
        if (!isLikeNone(ret)) {
            _assertNum(ret);
        }
        getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
        getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = getObject(arg0);
        const ret = typeof(val) === 'object' && val !== null;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
        const ret = getObject(arg0) in getObject(arg1);
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
        const ret = getObject(arg0) == getObject(arg1);
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbg_String_88810dfeb4021902 = function() { return logError(function (arg0, arg1) {
        const ret = String(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    }, arguments) };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_841ac57cff3d672b = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
    }, arguments) };
    imports.wbg.__wbg_create_e7a4871bd307ada9 = function() { return logError(function (arg0) {
        create(arg0 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_updatememory_acd4e8342aa8de4a = function() { return logError(function (arg0) {
        update_memory(takeObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_savetemplate_c4cab19a5b168471 = function() { return logError(function (arg0, arg1, arg2) {
        var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
        wasm.__wbindgen_free(arg0, arg1 * 4, 4);
        save_template(v0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_getnode_f20ae005be1aee22 = function() { return logError(function (arg0) {
        const ret = get_node(arg0 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_initilize_bfb3db58295295dd = function() { return logError(function (arg0, arg1) {
        initilize(takeObject(arg0), getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setAttributeInner_605314ec1c8cafdf = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        var v0 = getCachedStringFromWasm0(arg1, arg2);
        var v1 = getCachedStringFromWasm0(arg4, arg5);
        setAttributeInner(takeObject(arg0), v0, takeObject(arg3), v1);
    }, arguments) };
    imports.wbg.__wbg_new_abda76e883ba8a5f = function() { return logError(function () {
        const ret = new Error();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    }, arguments) };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function() { return logError(function (arg0, arg1) {
        var v0 = getCachedStringFromWasm0(arg0, arg1);
    if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
    console.error(v0);
}, arguments) };
imports.wbg.__wbg_Window_7bd5d737b6110ed5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_WorkerGlobalScope_10e1fa12a09a520b = function() { return logError(function (arg0) {
    const ret = getObject(arg0).WorkerGlobalScope;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_queueMicrotask_f61ee94ee663068b = function() { return logError(function (arg0) {
    queueMicrotask(getObject(arg0));
}, arguments) };
imports.wbg.__wbg_queueMicrotask_f82fc5d1e8f816ae = function() { return logError(function (arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_is_function = function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    _assertBoolean(ret);
    return ret;
};
imports.wbg.__wbg_crypto_d05b68a3572bb8ca = function() { return logError(function (arg0) {
    const ret = getObject(arg0).crypto;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_process_b02b3570280d0366 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).process;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_versions_c1cb42213cedf0f5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).versions;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_node_43b1089f407e4ec2 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).node;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_msCrypto_10fc94afee92bd76 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).msCrypto;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_require_9a7e0f667ead4995 = function() { return handleError(function () {
    const ret = module.require;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_getRandomValues_7e42b4fb8779dc6d = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).getRandomValues(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_randomFillSync_b70ccbdf4926a99d = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).randomFillSync(takeObject(arg1));
}, arguments) };
imports.wbg.__wbg_instanceof_Window_9029196b662bc42a = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_document_f7ace2b956f30a4f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_navigator_7c9103698acde322 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_devicePixelRatio_f9de7bddca0eaf20 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).devicePixelRatio;
    return ret;
}, arguments) };
imports.wbg.__wbg_requestAnimationFrame_d082200514b6674d = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_createElement_4891554b28d3388b = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createElement(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createElementNS_119acf9e82482041 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    const ret = getObject(arg0).createElementNS(v0, v1);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createTextNode_2fd22cd7e543f938 = function() { return logError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createTextNode(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_getElementById_cc0e0d931b0d9a28 = function() { return logError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getElementById(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_querySelectorAll_c03e8664a5a0f0c5 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).querySelectorAll(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_Element_4622f5da1249a3eb = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Element;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_getAttribute_3d8fcc9eaea35a17 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    const ret = getObject(arg1).getAttribute(v0);
    var ptr2 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len2;
    getInt32Memory0()[arg0 / 4 + 0] = ptr2;
}, arguments) };
imports.wbg.__wbg_getBoundingClientRect_ac9db8cf97ca8083 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).getBoundingClientRect();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_scrollIntoView_2ae69bbaf6ae4685 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).scrollIntoView(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_toggleAttribute_cd4962b3dd865542 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).toggleAttribute(v0);
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_instanceof_CompositionEvent_f079d7acac3bb64f = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof CompositionEvent;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_data_03708a776af7d2f6 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).data;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_width_e0c6b79d8cdd8897 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).width;
    return ret;
}, arguments) };
imports.wbg.__wbg_height_bed51746e072a118 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).height;
    return ret;
}, arguments) };
imports.wbg.__wbg_result_58251a5d230b00f6 = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).result;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_setonload_500a3ab3ebc0147b = function() { return logError(function (arg0, arg1) {
    getObject(arg0).onload = getObject(arg1);
}, arguments) };
imports.wbg.__wbg_new_9b551002cd49569b = function() { return handleError(function () {
    const ret = new FileReader();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_readAsArrayBuffer_07e155f1a3cd4ac2 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).readAsArrayBuffer(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_readAsText_52b21d5dd5a885e4 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).readAsText(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_features_dfb2178c91fa1dd7 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_limits_45ceb777867eb768 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_queue_f2aeb5c277e56f93 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).queue;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_setonuncapturederror_b3c814f611d5e585 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).onuncapturederror = getObject(arg1);
}, arguments) };
imports.wbg.__wbg_createBindGroup_fa5515d52f9c6a69 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createBindGroup(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createBindGroupLayout_af3b9d9ee0a1f5f9 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createBuffer_36e159f52cc644a7 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createBuffer(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createCommandEncoder_a50a1dab2b499b95 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createComputePipeline_89131452dfd12672 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createComputePipeline(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createPipelineLayout_1e10c8281fb85c01 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createQuerySet_ccb746122176f8e5 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createQuerySet(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createRenderBundleEncoder_ad2d0237f581427b = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createRenderPipeline_745f00bcb1ca6edf = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createSampler_09cd36835c9befb3 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createSampler(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createShaderModule_59bbf537b8b5cf7c = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createShaderModule(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createTexture_dbd00b550944125c = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createTexture(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_destroy_abb6deaa6cb27aa3 = function() { return logError(function (arg0) {
    getObject(arg0).destroy();
}, arguments) };
imports.wbg.__wbg_popErrorScope_19075fb98a08b740 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).popErrorScope();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_pushErrorScope_0728aae3f2d3ed48 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).pushErrorScope(takeObject(arg1));
}, arguments) };
imports.wbg.__wbg_pointerId_701aab7b4fb073ff = function() { return logError(function (arg0) {
    const ret = getObject(arg0).pointerId;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_width_ff9524f9a20fa31b = function() { return logError(function (arg0) {
    const ret = getObject(arg0).width;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_height_f6953361ca39cf59 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).height;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_pressure_e388b6fd623a3917 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).pressure;
    return ret;
}, arguments) };
imports.wbg.__wbg_tangentialPressure_0dbdc7061588dff6 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).tangentialPressure;
    return ret;
}, arguments) };
imports.wbg.__wbg_tiltX_edd44454d780d537 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).tiltX;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_tiltY_b4cb8c98b666ec9d = function() { return logError(function (arg0) {
    const ret = getObject(arg0).tiltY;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_twist_0acb3c0a8d7491d5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).twist;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_pointerType_0009b1e4e6b0f428 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).pointerType;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_isPrimary_5b023a7fb7fa8716 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).isPrimary;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_deltaX_84508d00a1050e70 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).deltaX;
    return ret;
}, arguments) };
imports.wbg.__wbg_deltaY_64823169afb0335d = function() { return logError(function (arg0) {
    const ret = getObject(arg0).deltaY;
    return ret;
}, arguments) };
imports.wbg.__wbg_deltaZ_0b63b6d98ff75513 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).deltaZ;
    return ret;
}, arguments) };
imports.wbg.__wbg_deltaMode_1c680147cfdba8a5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).deltaMode;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_end_bdfb66792e0c59a2 = function() { return logError(function (arg0) {
    getObject(arg0).end();
}, arguments) };
imports.wbg.__wbg_executeBundles_0a1fdfd83c1a3e57 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).executeBundles(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_setBlendConstant_e89574db5137b2f6 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).setBlendConstant(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_setScissorRect_0af8c89e90a6e89c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
}, arguments) };
imports.wbg.__wbg_setStencilReference_71be0db67db2f7ab = function() { return logError(function (arg0, arg1) {
    getObject(arg0).setStencilReference(arg1 >>> 0);
}, arguments) };
imports.wbg.__wbg_setViewport_9c5fb686baf1cf4f = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
}, arguments) };
imports.wbg.__wbg_setBindGroup_ed098a3302f084a7 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
}, arguments) };
imports.wbg.__wbg_setBindGroup_ce4432036922cd83 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
}, arguments) };
imports.wbg.__wbg_draw_6357a5fbc8a6b097 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
}, arguments) };
imports.wbg.__wbg_drawIndexed_5d1dd89d7375148c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
}, arguments) };
imports.wbg.__wbg_drawIndexedIndirect_526599171cfbbee5 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
}, arguments) };
imports.wbg.__wbg_drawIndirect_8dd595dc622e21ac = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
}, arguments) };
imports.wbg.__wbg_setIndexBuffer_1f4a86d1cc8c16d9 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
}, arguments) };
imports.wbg.__wbg_setIndexBuffer_9f8493460611f96b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
}, arguments) };
imports.wbg.__wbg_setPipeline_18ce556bdea62cc5 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_setVertexBuffer_2a2c84d65c1063f9 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
}, arguments) };
imports.wbg.__wbg_setVertexBuffer_176c2dff823c42c1 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
}, arguments) };
imports.wbg.__wbg_has_8720889cf3ad610c = function() { return logError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).has(v0);
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlInputElement_31b50e0cf542c524 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLInputElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_checked_5ccb3a66eb054121 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).checked;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_files_0b91078a034a0f7b = function() { return logError(function (arg0) {
    const ret = getObject(arg0).files;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_type_0f4fee5293059bbf = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).type;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_value_9423da9d988ee8cf = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_getPreferredCanvasFormat_1f6c9ef810196b92 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).getPreferredCanvasFormat();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_requestAdapter_d8298d7a27a391f0 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAdapter(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlElement_6f4725d4677c7968 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_blur_53431c003c82bf53 = function() { return handleError(function (arg0) {
    getObject(arg0).blur();
}, arguments) };
imports.wbg.__wbg_focus_dbcbbbb2a04c0e1f = function() { return handleError(function (arg0) {
    getObject(arg0).focus();
}, arguments) };
imports.wbg.__wbg_new_9d3e795dcd24a5d9 = function() { return handleError(function (arg0) {
    const ret = new ResizeObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_observe_e3e06d8e2ad2b89c = function() { return logError(function (arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_dispatchWorkgroups_c484cd3530a3801d = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
}, arguments) };
imports.wbg.__wbg_dispatchWorkgroupsIndirect_2b89ee1731fab5f8 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
}, arguments) };
imports.wbg.__wbg_end_dab719019df5969c = function() { return logError(function (arg0) {
    getObject(arg0).end();
}, arguments) };
imports.wbg.__wbg_setPipeline_598117fdeb73cf8f = function() { return logError(function (arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_setBindGroup_c619f49c16ef095b = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
}, arguments) };
imports.wbg.__wbg_setBindGroup_dffce83253968cdd = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
}, arguments) };
imports.wbg.__wbg_getBindGroupLayout_20dc45d52b96fa42 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_GpuValidationError_af2aa2e306669317 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUValidationError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_altKey_612289acf855835c = function() { return logError(function (arg0) {
    const ret = getObject(arg0).altKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_ctrlKey_582686fb2263dd3c = function() { return logError(function (arg0) {
    const ret = getObject(arg0).ctrlKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_shiftKey_48e8701355d8e2d4 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).shiftKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_metaKey_43193b7cc99f8914 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).metaKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_location_e5f8d98ba89b596e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).location;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_repeat_52850ed66db69aba = function() { return logError(function (arg0) {
    const ret = getObject(arg0).repeat;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_key_8aeaa079126a9cc7 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).key;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_code_96d6322b968b2d17 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).code;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_gpu_1678673f109c8aeb = function() { return logError(function (arg0) {
    const ret = getObject(arg0).gpu;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createView_3e46af1f54fdcd1f = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createView(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_destroy_728f676d96e34538 = function() { return logError(function (arg0) {
    getObject(arg0).destroy();
}, arguments) };
imports.wbg.__wbg_gpu_24536c9523d924b1 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).gpu;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_GpuAdapter_c0a5a310603ba618 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUAdapter;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_features_88901f43932fb28e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_limits_a7f3fbf58768b61f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_requestDevice_068e794820eb88eb = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).requestDevice(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_copyExternalImageToTexture_819ec294d299f624 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };
imports.wbg.__wbg_submit_3104e9b014f75846 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).submit(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_writeBuffer_becf0c8f0323ffd7 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
}, arguments) };
imports.wbg.__wbg_writeTexture_465ecc6146e5052c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
}, arguments) };
imports.wbg.__wbg_finish_863657abae52896e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_finish_e580ef236d53f04b = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_setBindGroup_e6d2dd2ab3573b6d = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
}, arguments) };
imports.wbg.__wbg_setBindGroup_6bc8944422dbb3cd = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
}, arguments) };
imports.wbg.__wbg_draw_3958097471a10642 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
}, arguments) };
imports.wbg.__wbg_drawIndexed_8856cc4ccffa3498 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
}, arguments) };
imports.wbg.__wbg_drawIndexedIndirect_0404fa6cb9a6db25 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
}, arguments) };
imports.wbg.__wbg_drawIndirect_95c6eb1494a44d06 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
}, arguments) };
imports.wbg.__wbg_setIndexBuffer_4dc5432dc348458d = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
}, arguments) };
imports.wbg.__wbg_setIndexBuffer_f3bae4da9e407eaf = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
}, arguments) };
imports.wbg.__wbg_setPipeline_66f1e900256fc946 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_setVertexBuffer_c782d133fd439184 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
}, arguments) };
imports.wbg.__wbg_setVertexBuffer_4da0a96267ce82db = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
}, arguments) };
imports.wbg.__wbg_debug_9a6b3243fbbebb61 = function() { return logError(function (arg0) {
    console.debug(getObject(arg0));
}, arguments) };
imports.wbg.__wbg_error_788ae33f81d3b84b = function() { return logError(function (arg0) {
    console.error(getObject(arg0));
}, arguments) };
imports.wbg.__wbg_info_2e30e8204b29d91d = function() { return logError(function (arg0) {
    console.info(getObject(arg0));
}, arguments) };
imports.wbg.__wbg_log_1d3ae0273d8f4f8a = function() { return logError(function (arg0) {
    console.log(getObject(arg0));
}, arguments) };
imports.wbg.__wbg_warn_d60e832f9882c1b2 = function() { return logError(function (arg0) {
    console.warn(getObject(arg0));
}, arguments) };
imports.wbg.__wbg_name_a46b2d975591a0b3 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_length_b941879633a63ad8 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).length;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_item_77ac9b6a3db8c30a = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).item(arg1 >>> 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_message_c934153af8567cdb = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_setwidth_15266a5e81f43cf0 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
}, arguments) };
imports.wbg.__wbg_setheight_2e9bab573f1775a6 = function() { return logError(function (arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
}, arguments) };
imports.wbg.__wbg_instanceof_ResizeObserverEntry_4cc5bd5c64152452 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ResizeObserverEntry;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_contentRect_3a38b9ecf8994843 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).contentRect;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_navigator_41bd88b80ed4685e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_size_6540ddb49e0d7120 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).size;
    return ret;
}, arguments) };
imports.wbg.__wbg_usage_f5b34f3e0170424b = function() { return logError(function (arg0) {
    const ret = getObject(arg0).usage;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_destroy_9b5398e5b148e210 = function() { return logError(function (arg0) {
    getObject(arg0).destroy();
}, arguments) };
imports.wbg.__wbg_getMappedRange_becef7e3d9dc5489 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getMappedRange(arg1, arg2);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_mapAsync_91acdcf41b7ae21d = function() { return logError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_unmap_1677c09514e08e64 = function() { return logError(function (arg0) {
    getObject(arg0).unmap();
}, arguments) };
imports.wbg.__wbg_label_c7970304720cf8b0 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).label;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_beginComputePass_579a2563c561da68 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).beginComputePass(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_beginRenderPass_d04327f7231bd5af = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).beginRenderPass(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_clearBuffer_c370e7adb8398388 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2);
}, arguments) };
imports.wbg.__wbg_clearBuffer_b8e6751290709d43 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
}, arguments) };
imports.wbg.__wbg_copyBufferToBuffer_79ac12f409453cf0 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
}, arguments) };
imports.wbg.__wbg_copyBufferToTexture_ac956e6d47c24e73 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };
imports.wbg.__wbg_copyTextureToBuffer_787ec8d8c4c216f1 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };
imports.wbg.__wbg_copyTextureToTexture_a86e849469b0ef38 = function() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };
imports.wbg.__wbg_finish_5153789564a5eee5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_finish_d1049a13335e8326 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_resolveQuerySet_8ac49c71e15cdf6a = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
}, arguments) };
imports.wbg.__wbg_writeTimestamp_107647519ce52436 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).writeTimestamp(getObject(arg1), arg2 >>> 0);
}, arguments) };
imports.wbg.__wbg_instanceof_GpuOutOfMemoryError_45166ef4e2774fbe = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUOutOfMemoryError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_getBindGroupLayout_dfc1b97f78c04beb = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_maxTextureDimension1D_4d1ddb46ed9dc470 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureDimension1D;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxTextureDimension2D_37a46e61490c8297 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureDimension2D;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxTextureDimension3D_7e3a97204d211743 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureDimension3D;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxTextureArrayLayers_fee4db585706a5eb = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureArrayLayers;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxBindGroups_dc8a5f97ba653c91 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxBindGroups;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxBindingsPerBindGroup_3d5ab311420be5df = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxBindingsPerBindGroup;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxDynamicUniformBuffersPerPipelineLayout_6b839b7dc97f34f0 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxDynamicStorageBuffersPerPipelineLayout_5328cd2b9d884831 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxSampledTexturesPerShaderStage_ac006b00cf776b4a = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxSamplersPerShaderStage_dc092d6a272be20a = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxSamplersPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxStorageBuffersPerShaderStage_dc5b58734b9ab932 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxStorageTexturesPerShaderStage_2fec939cb0d5bbfd = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxUniformBuffersPerShaderStage_b30d53cbf89caeae = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxUniformBufferBindingSize_eec576e1342504b5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxUniformBufferBindingSize;
    return ret;
}, arguments) };
imports.wbg.__wbg_maxStorageBufferBindingSize_1ef0cc5e43dad09b = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxStorageBufferBindingSize;
    return ret;
}, arguments) };
imports.wbg.__wbg_minUniformBufferOffsetAlignment_3af8c32faa30c5d8 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).minUniformBufferOffsetAlignment;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_minStorageBufferOffsetAlignment_766ef8ea8f9fe6e1 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).minStorageBufferOffsetAlignment;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxVertexBuffers_b4d31be9e3f93990 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxVertexBuffers;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxBufferSize_2d8398a691b9a8ce = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxBufferSize;
    return ret;
}, arguments) };
imports.wbg.__wbg_maxVertexAttributes_904c5eb19a6f6c65 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxVertexAttributes;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxVertexBufferArrayStride_6800975c373d83bc = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxVertexBufferArrayStride;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxInterStageShaderComponents_b9f179b1cde06d08 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxInterStageShaderComponents;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxComputeWorkgroupStorageSize_9318e498283b79fb = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxComputeInvocationsPerWorkgroup_2bfea723194ac5a0 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxComputeWorkgroupSizeX_91fc9ba04de4148f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeX;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxComputeWorkgroupSizeY_9052627dce4a7d1f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeY;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxComputeWorkgroupSizeZ_45a1a82f8446a750 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_maxComputeWorkgroupsPerDimension_100ee7392cc04c20 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_error_f85e77a2651e41dc = function() { return logError(function (arg0) {
    const ret = getObject(arg0).error;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlCanvasElement_da5f9efa0688cf6d = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLCanvasElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_width_2931aaedd21f1fff = function() { return logError(function (arg0) {
    const ret = getObject(arg0).width;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_setwidth_a667a942dba6656e = function() { return logError(function (arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
}, arguments) };
imports.wbg.__wbg_height_0d36fbbeb60b0661 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).height;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_setheight_a747d440760fe5aa = function() { return logError(function (arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
}, arguments) };
imports.wbg.__wbg_getContext_7c5944ea807bf5d3 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getContext(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlSelectElement_75d8a9ac3b088f08 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLSelectElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_value_c45528fab757534f = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlTextAreaElement_348d0e222e16eec4 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLTextAreaElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_value_3c5f08ffc2b7d6f9 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_propertyName_bc5f849981f1e91b = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).propertyName;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_elapsedTime_4a5788bafe903a63 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).elapsedTime;
    return ret;
}, arguments) };
imports.wbg.__wbg_pseudoElement_b1865fc629d586c8 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).pseudoElement;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_animationName_96cb6c08f1125be6 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).animationName;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_elapsedTime_fe2486e8422a9ac7 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).elapsedTime;
    return ret;
}, arguments) };
imports.wbg.__wbg_pseudoElement_4b7a498b190ca9cf = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).pseudoElement;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_width_5728ff70a76f27ac = function() { return logError(function (arg0) {
    const ret = getObject(arg0).width;
    return ret;
}, arguments) };
imports.wbg.__wbg_height_7fd80bb9bbf69d8c = function() { return logError(function (arg0) {
    const ret = getObject(arg0).height;
    return ret;
}, arguments) };
imports.wbg.__wbg_top_98ff0408c018d25e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).top;
    return ret;
}, arguments) };
imports.wbg.__wbg_left_23a613d619fb4206 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).left;
    return ret;
}, arguments) };
imports.wbg.__wbg_type_4197dff653b7d208 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).type;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_target_f171e89c61e2bccf = function() { return logError(function (arg0) {
    const ret = getObject(arg0).target;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_preventDefault_24104f3f0a54546a = function() { return logError(function (arg0) {
    getObject(arg0).preventDefault();
}, arguments) };
imports.wbg.__wbg_instanceof_GpuCanvasContext_7a77e275c38d41d8 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUCanvasContext;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_configure_93a57a4e5e0f8bcf = function() { return logError(function (arg0, arg1) {
    getObject(arg0).configure(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_getCurrentTexture_ecedc4f6f71990d2 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).getCurrentTexture();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlFormElement_b57527983c7c1ada = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLFormElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_screenX_90d9e75d4db9ae09 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).screenX;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_screenY_80d720b27c3268de = function() { return logError(function (arg0) {
    const ret = getObject(arg0).screenY;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_clientX_1a480606ab0cabaa = function() { return logError(function (arg0) {
    const ret = getObject(arg0).clientX;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_clientY_9c7878f7faf3900f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).clientY;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_offsetX_5a58f16f6c3a41b6 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).offsetX;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_offsetY_c45b4956f6429a95 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).offsetY;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_ctrlKey_0a805df688b5bf42 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).ctrlKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_shiftKey_8a070ab6169b5fa4 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).shiftKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_altKey_6fc1761a6b7a406e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).altKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_metaKey_d89287be4389a3c1 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).metaKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_button_7a095234b69de930 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).button;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_buttons_d0f40e1650e3fa28 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).buttons;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_instanceof_Node_cffd9c3b74760745 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Node;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_parentElement_c75962bc9997ea5f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).parentElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_textContent_c5d9e21ee03c63d4 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).textContent;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_appendChild_51339d4cde00ee22 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_get_c77649dd3862b63a = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_altKey_536428fa8344c5f0 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).altKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_metaKey_3c4655f73129d59f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).metaKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_ctrlKey_7124cf47f48ae0ea = function() { return logError(function (arg0) {
    const ret = getObject(arg0).ctrlKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_shiftKey_66f6a9792f554cb8 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).shiftKey;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_pageX_3e6c12486830d4c5 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).pageX;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_pageY_930d5bb1af089ee4 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).pageY;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_charCodeAt_dda38fa2b2e855eb = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).charCodeAt(arg1 >>> 0);
    return ret;
}, arguments) };
imports.wbg.__wbg_get_0ee8ea3c7c984c45 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_length_161c0d89c6535c1d = function() { return logError(function (arg0) {
    const ret = getObject(arg0).length;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_new_75208e29bddfd88c = function() { return logError(function () {
    const ret = new Array();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_newnoargs_cfecb3965268594c = function() { return logError(function (arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Function(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_new_d1cc518eff6805bb = function() { return logError(function () {
    const ret = new Map();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_next_586204376d2ed373 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_next_b2d3366343a208b3 = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_done_90b14d6f6eacc42f = function() { return logError(function (arg0) {
    const ret = getObject(arg0).done;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_value_3158be908c80a75e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_iterator_40027cdd598da26b = function() { return logError(function () {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_get_3fddfed2c83f434c = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_call_3f093dd26d5569f8 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_new_632630b5cec17f21 = function() { return logError(function () {
    const ret = new Object();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_length_ceaac4086e667d58 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).length;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_self_05040bd9523805b9 = function() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_window_adc720039f2cb14f = function() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_globalThis_622105db80c1457d = function() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_global_f56b013ed9bcf359 = function() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_at_167bddce09cf1499 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).at(arg1);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_set_79c308ecd9a1d091 = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
}, arguments) };
imports.wbg.__wbg_isArray_e783c41d0dd19b44 = function() { return logError(function (arg0) {
    const ret = Array.isArray(getObject(arg0));
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_push_0239ee92f127e807 = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_instanceof_ArrayBuffer_9221fa854ffb71b5 = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_newwithargs_a6c3c567747cd30b = function() { return logError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    var v1 = getCachedStringFromWasm0(arg2, arg3);
    const ret = new Function(v0, v1);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_call_67f2111acd2dfdb6 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_set_e4cfc2763115ffc7 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_entries_8d2edb6177b49770 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).entries();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_isSafeInteger_a23a66ee7c41b273 = function() { return logError(function (arg0) {
    const ret = Number.isSafeInteger(getObject(arg0));
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_instanceof_Object_4abbcd5d20d5f7df = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Object;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_entries_488960b196cfb6a5 = function() { return logError(function (arg0) {
    const ret = Object.entries(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_valueOf_524ec9f8b05882e8 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).valueOf();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_resolve_5da6faf2c96fd1d5 = function() { return logError(function (arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_then_f9e58f5a50f43eae = function() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_then_20a5920e447d1cb1 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_buffer_b914fb8b50ebbc3e = function() { return logError(function (arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_newwithbyteoffsetandlength_0de9ee56e9f6ee6e = function() { return logError(function (arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_new_b1f2d6842d615181 = function() { return logError(function (arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_set_7d988c98e6ced92d = function() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
}, arguments) };
imports.wbg.__wbg_length_21c4b0ae73cba59d = function() { return logError(function (arg0) {
    const ret = getObject(arg0).length;
    _assertNum(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_instanceof_Uint8Array_c299a4ee232e76ba = function() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_newwithlength_0d03cef43b68a530 = function() { return logError(function (arg0) {
    const ret = new Uint8Array(arg0 >>> 0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_buffer_67e624f5a0ab2319 = function() { return logError(function (arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_subarray_adc418253d76e2f1 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_set_961700853a212a39 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    _assertBoolean(ret);
    return ret;
}, arguments) };
imports.wbg.__wbg_stringify_865daa6fb8c83d5a = function() { return handleError(function (arg0) {
    const ret = JSON.stringify(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
    const v = getObject(arg1);
    const ret = typeof(v) === 'bigint' ? v : undefined;
    if (!isLikeNone(ret)) {
        _assertBigInt(ret);
    }
    getBigInt64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? BigInt(0) : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
};
imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};
imports.wbg.__wbindgen_memory = function() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper389 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 44, __wbg_adapter_48);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper390 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeClosure(arg0, arg1, 44, __wbg_adapter_51);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper627 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 202, __wbg_adapter_54);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper628 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 202, __wbg_adapter_57);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper631 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 202, __wbg_adapter_60);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper1061 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 432, __wbg_adapter_63);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper1063 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 432, __wbg_adapter_63);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_closure_wrapper1085 = function() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 439, __wbg_adapter_68);
    return addHeapObject(ret);
}, arguments) };
imports['./snippets/dioxus-interpreter-js-85a0a2f8acce2e86/inline0.js'] = __wbg_star0;

return imports;
}

function __wbg_init_memory(imports, maybe_memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedBigInt64Memory0 = null;
    cachedFloat64Memory0 = null;
    cachedInt32Memory0 = null;
    cachedUint32Memory0 = null;
    cachedUint8Memory0 = null;

    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(input) {
    if (wasm !== undefined) return wasm;


    const imports = __wbg_get_imports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await input, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync }
export default __wbg_init;
