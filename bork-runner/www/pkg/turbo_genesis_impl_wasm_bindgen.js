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

let WASM_VECTOR_LEN = 0;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
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

    if (typeof(arg) !== 'string') throw new Error(`expected a string argument, found ${typeof(arg)}`);

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

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
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error(`expected a boolean argument, found ${typeof(n)}`);
    }
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error(`expected a number argument, found ${typeof(n)}`);
}

function _assertBigInt(n) {
    if (typeof(n) !== 'bigint') throw new Error(`expected a bigint argument, found ${typeof(n)}`);
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

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

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
                CLOSURE_DTORS.unregister(state);
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
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
function __wbg_adapter_44(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h161c54fd178bf91d(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
}

function __wbg_adapter_47(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    wasm._dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2ae8ed5115181fdd(arg0, arg1, arg2, arg3);
}

function __wbg_adapter_50(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    const ret = wasm._dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf6dcca4c44ccf724(arg0, arg1, arg2, arg3);
    return ret;
}

function __wbg_adapter_53(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    _assertNum(arg10);
    _assertNum(arg11);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0a49c8f77d43b3d6(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);
    return ret >>> 0;
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
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_56(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h12ac25ec3fa362e6(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_61(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    const ret = wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h04c9fb4110131864(arg0, arg1);
    return BigInt.asUintN(64, ret);
}

function __wbg_adapter_64(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0e0bacd9500fee95(arg0, arg1);
}

function __wbg_adapter_67(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he42895901169c1c9(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

function __wbg_adapter_70(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertBigInt(arg7);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3c7680a3352c13e9(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7);
    return ret;
}

function __wbg_adapter_75(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h05eb9c53ee2e00b4(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7);
}

function __wbg_adapter_78(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertBigInt(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdd9bd477561aa61d(arg0, arg1, arg2, arg3, arg4, arg5, arg6);
    return ret >>> 0;
}

function __wbg_adapter_81(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h36ae4b302def00bd(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
    return ret;
}

function __wbg_adapter_84(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11, arg12, arg13) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertBigInt(arg2);
    _assertBigInt(arg3);
    _assertBigInt(arg4);
    _assertBigInt(arg5);
    _assertBigInt(arg6);
    _assertBigInt(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    _assertNum(arg10);
    _assertBigInt(arg11);
    _assertNum(arg12);
    _assertNum(arg13);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J_K_L___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha7b44fff6ce86f15(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11, arg12, arg13);
}

function __wbg_adapter_87(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7cfb047f9aa6b2ac(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
}

function __wbg_adapter_92(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    const ret = wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1fda849736229213(arg0, arg1);
    return ret >>> 0;
}

function __wbg_adapter_95(arg0, arg1, arg2, arg3, arg4) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__Fn__A_B_C___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbb720ec6edcd4f5f(arg0, arg1, arg2, arg3, arg4);
}

function __wbg_adapter_98(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    const ret = wasm._dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c3144be7a60a728(arg0, arg1, arg2);
    return ret;
}

function __wbg_adapter_101(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    _assertNum(arg10);
    _assertNum(arg11);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0a49c8f77d43b3d6(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);
    return ret;
}

function __wbg_adapter_104(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb00baa4ac20ddb87(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
    return ret;
}

function __wbg_adapter_107(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    const ret = wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1fda849736229213(arg0, arg1);
    return ret;
}

function __wbg_adapter_110(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    wasm._dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha20d4f629fac8293(arg0, arg1, arg2);
}

function __wbg_adapter_113(arg0, arg1, arg2, arg3, arg4, arg5) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    wasm._dyn_core__ops__function__Fn__A_B_C_D___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hfd30a9336f657bda(arg0, arg1, arg2, arg3, arg4, arg5);
}

function __wbg_adapter_116(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertBigInt(arg2);
    _assertBigInt(arg3);
    _assertBigInt(arg4);
    _assertBigInt(arg5);
    _assertBigInt(arg6);
    _assertNum(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    _assertBigInt(arg10);
    _assertNum(arg11);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdc147d6638f1275f(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);
}

function __wbg_adapter_119(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    wasm._dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2ae8ed5115181fdd(arg0, arg1, arg2, arg3);
}

function __wbg_adapter_124(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h399dd21fa179106a(arg0, arg1, arg2, arg3, arg4, arg5, arg6);
}

function __wbg_adapter_127(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg7);
    _assertNum(arg8);
    _assertNum(arg9);
    _assertNum(arg10);
    wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h772b66740572b8d1(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10);
}

function __wbg_adapter_130(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg2);
    _assertNum(arg3);
    _assertNum(arg4);
    _assertNum(arg5);
    _assertNum(arg6);
    _assertNum(arg7);
    const ret = wasm._dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hccef0dcde61e042d(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7);
    return ret >>> 0;
}

function __wbg_adapter_137(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1691d30387bd8462(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_140(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdd6c5ec5a8be9079(arg0, arg1);
}

function __wbg_adapter_153(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb9dd283590838b58(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_158(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h22df361fb6088299(arg0, arg1, addHeapObject(arg2));
}

/**
* @param {Uint8Array} bytes
* @returns {TurboFileV0Contents}
*/
export function decode_turbofile_v0_contents(bytes) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.decode_turbofile_v0_contents(retptr, addHeapObject(bytes));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return TurboFileV0Contents.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}
/**
* @param {TurboFileV0Contents} contents
* @returns {Promise<TurboApp>}
*/
export function create_app_from_v0_contents(contents) {
    _assertClass(contents, TurboFileV0Contents);
    if (contents.__wbg_ptr === 0) {
        throw new Error('Attempt to use a moved value');
    }
    var ptr0 = contents.__destroy_into_raw();
    const ret = wasm.create_app_from_v0_contents(ptr0);
    return takeObject(ret);
}

/**
* @param {HTMLCanvasElement} canvas
* @param {TurboFileV0Contents} v0_contents
* @returns {Promise<void>}
*/
export function run(canvas, v0_contents) {
    _assertClass(v0_contents, TurboFileV0Contents);
    if (v0_contents.__wbg_ptr === 0) {
        throw new Error('Attempt to use a moved value');
    }
    var ptr0 = v0_contents.__destroy_into_raw();
    const ret = wasm.run(addHeapObject(canvas), ptr0);
    return takeObject(ret);
}

let cachedUint32ArrayMemory0 = null;

function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedInt32ArrayMemory0 = null;

function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

let cachedFloat32ArrayMemory0 = null;

function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addHeapObject(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

let stack_pointer = 128;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}
/**
* Handler for `console.log` invocations.
*
* If a test is currently running it takes the `args` array and stringifies
* it and appends it to the current output of the test. Otherwise it passes
* the arguments to the original `console.log` function, psased as
* `original`.
* @param {Array<any>} args
*/
export function __wbgtest_console_log(args) {
    try {
        wasm.__wbgtest_console_log(addBorrowedObject(args));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

/**
* Handler for `console.debug` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_debug(args) {
    try {
        wasm.__wbgtest_console_debug(addBorrowedObject(args));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

/**
* Handler for `console.info` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_info(args) {
    try {
        wasm.__wbgtest_console_info(addBorrowedObject(args));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

/**
* Handler for `console.warn` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_warn(args) {
    try {
        wasm.__wbgtest_console_warn(addBorrowedObject(args));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

/**
* Handler for `console.error` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_error(args) {
    try {
        wasm.__wbgtest_console_error(addBorrowedObject(args));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

function __wbg_adapter_1264(arg0, arg1, arg2, arg3, arg4) {
    _assertNum(arg0);
    _assertNum(arg1);
    _assertNum(arg3);
    wasm.wasm_bindgen__convert__closures__invoke3_mut__h70ff84634a24fb2b(arg0, arg1, addHeapObject(arg2), arg3, addHeapObject(arg4));
}

function __wbg_adapter_1317(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h5345ea8aea305903(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

const TurboAppFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_turboapp_free(ptr >>> 0, 1));
/**
*/
export class TurboApp {

    constructor() {
        throw new Error('cannot invoke `new` directly');
    }

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TurboApp.prototype);
        obj.__wbg_ptr = ptr;
        TurboAppFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TurboAppFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_turboapp_free(ptr, 0);
    }
}

const TurboFileV0ContentsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_turbofilev0contents_free(ptr >>> 0, 1));
/**
*/
export class TurboFileV0Contents {

    constructor() {
        throw new Error('cannot invoke `new` directly');
    }

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TurboFileV0Contents.prototype);
        obj.__wbg_ptr = ptr;
        TurboFileV0ContentsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TurboFileV0ContentsFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_turbofilev0contents_free(ptr, 0);
    }
}

const WasmBindgenTestContextFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmbindgentestcontext_free(ptr >>> 0, 1));
/**
* Runtime test harness support instantiated in JS.
*
* The node.js entry script instantiates a `Context` here which is used to
* drive test execution.
*/
export class WasmBindgenTestContext {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmBindgenTestContextFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmbindgentestcontext_free(ptr, 0);
    }
    /**
    * Creates a new context ready to run tests.
    *
    * A `Context` is the main structure through which test execution is
    * coordinated, and this will collect output and results for all executed
    * tests.
    */
    constructor() {
        const ret = wasm.wasmbindgentestcontext_new();
        this.__wbg_ptr = ret >>> 0;
        WasmBindgenTestContextFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
    * Inform this context about runtime arguments passed to the test
    * harness.
    * @param {any[]} args
    */
    args(args) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArrayJsValueToWasm0(args, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmbindgentestcontext_args(this.__wbg_ptr, ptr0, len0);
    }
    /**
    * Executes a list of tests, returning a promise representing their
    * eventual completion.
    *
    * This is the main entry point for executing tests. All the tests passed
    * in are the JS `Function` object that was plucked off the
    * `WebAssembly.Instance` exports list.
    *
    * The promise returned resolves to either `true` if all tests passed or
    * `false` if at least one test failed.
    * @param {any[]} tests
    * @returns {Promise<any>}
    */
    run(tests) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArrayJsValueToWasm0(tests, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbindgentestcontext_run(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
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
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
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
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_log_4a8b5103d5c659fb = function() { return logError(function (arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_error_fc520cec86426345 = function() { return logError(function (arg0, arg1) {
        console.error(getStringFromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_turboapp_new = function() { return logError(function (arg0) {
        const ret = TurboApp.__wrap(arg0);
        return addHeapObject(ret);
    }, arguments) };
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
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
        const ret = BigInt.asUintN(64, arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = getObject(arg0) === getObject(arg1);
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = getObject(arg0);
        const ret = typeof(val) === 'object' && val !== null;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
        const ret = getObject(arg0) in getObject(arg1);
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'number' ? obj : undefined;
        if (!isLikeNone(ret)) {
            _assertNum(ret);
        }
        getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'function';
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
        const ret = getObject(arg0) == getObject(arg1);
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getwithrefkey_5e6d9547403deab8 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0)[getObject(arg1)];
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_offsetX_d08eda91526f22a2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).offsetX;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_offsetY_3c895bb1534dfbf4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).offsetY;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_webkitFullscreenElement_533c5f32e2ac8d0c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).webkitFullscreenElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_onpointerrawupdate_e087759b4021ec00 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).onpointerrawupdate;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getCoalescedEvents_4665669d237be577 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCoalescedEvents;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestFullscreen_a851d70cb190396a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).requestFullscreen;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_scheduler_6932606c19435996 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).scheduler;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestIdleCallback_081ddac93612a53e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).requestIdleCallback;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_webkitRequestFullscreen_8abcfecec7127495 = function() { return logError(function (arg0) {
        getObject(arg0).webkitRequestFullscreen();
    }, arguments) };
    imports.wbg.__wbg_requestFullscreen_f4349fb8a7429cf9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).requestFullscreen();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_scheduler_8082c844a9cfc0df = function() { return logError(function (arg0) {
        const ret = getObject(arg0).scheduler;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_postTask_4674878f9a603824 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).postTask(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_prototype_8e5075a5dd95f801 = function() { return logError(function () {
        const ret = ResizeObserverEntry.prototype;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_Window_cc0273a5da2c36dc = function() { return logError(function (arg0) {
        const ret = getObject(arg0).Window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_performance_eeefc685c9bc38b4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).performance;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_now_e0d8ec93dd25766a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).now();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuCanvasContext_a8f6b2929cf7ac72 = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_instanceof_GpuValidationError_5da6ac919fba8737 = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_instanceof_GpuAdapter_675bbcd7cd565366 = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_configure_3d097f5e85c1b8be = function() { return logError(function (arg0, arg1) {
        getObject(arg0).configure(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_getCurrentTexture_708f1004c7e69d9a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCurrentTexture();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_label_c9198cdb1825abd5 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).label;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_beginComputePass_90bd231e6b9ce199 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).beginComputePass(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_beginRenderPass_7584717956df77f1 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).beginRenderPass(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_clearBuffer_ecf35ab0c911f925 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).clearBuffer(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_clearBuffer_52403064b3cc4cfb = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
    }, arguments) };
    imports.wbg.__wbg_copyBufferToBuffer_b600364aa4c31d9a = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_copyBufferToTexture_4559e6d203d55ea4 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_copyTextureToBuffer_16d76ec4aa2ee7d5 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_copyTextureToTexture_f00441e7f7ae92ba = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_finish_07611ae5e5c28379 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).finish();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_finish_d8e0cb06cfd278af = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).finish(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_resolveQuerySet_c9db96541b4a0f9d = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_writeTimestamp_c8bbe7180194237d = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).writeTimestamp(getObject(arg1), arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_has_6dc604737cc778ea = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).has(getStringFromWasm0(arg1, arg2));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_message_f2a2e76018f5d52f = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).message;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuOutOfMemoryError_fc6bfc92523aa4f4 = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_getMappedRange_ba391bfb20f2a393 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getMappedRange(arg1, arg2);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getPreferredCanvasFormat_45283b0bce3a7bda = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getPreferredCanvasFormat();
        return {"r8unorm":0,"r8snorm":1,"r8uint":2,"r8sint":3,"r16uint":4,"r16sint":5,"r16float":6,"rg8unorm":7,"rg8snorm":8,"rg8uint":9,"rg8sint":10,"r32uint":11,"r32sint":12,"r32float":13,"rg16uint":14,"rg16sint":15,"rg16float":16,"rgba8unorm":17,"rgba8unorm-srgb":18,"rgba8snorm":19,"rgba8uint":20,"rgba8sint":21,"bgra8unorm":22,"bgra8unorm-srgb":23,"rgb9e5ufloat":24,"rgb10a2unorm":25,"rg11b10ufloat":26,"rg32uint":27,"rg32sint":28,"rg32float":29,"rgba16uint":30,"rgba16sint":31,"rgba16float":32,"rgba32uint":33,"rgba32sint":34,"rgba32float":35,"stencil8":36,"depth16unorm":37,"depth24plus":38,"depth24plus-stencil8":39,"depth32float":40,"depth32float-stencil8":41,"bc1-rgba-unorm":42,"bc1-rgba-unorm-srgb":43,"bc2-rgba-unorm":44,"bc2-rgba-unorm-srgb":45,"bc3-rgba-unorm":46,"bc3-rgba-unorm-srgb":47,"bc4-r-unorm":48,"bc4-r-snorm":49,"bc5-rg-unorm":50,"bc5-rg-snorm":51,"bc6h-rgb-ufloat":52,"bc6h-rgb-float":53,"bc7-rgba-unorm":54,"bc7-rgba-unorm-srgb":55,"etc2-rgb8unorm":56,"etc2-rgb8unorm-srgb":57,"etc2-rgb8a1unorm":58,"etc2-rgb8a1unorm-srgb":59,"etc2-rgba8unorm":60,"etc2-rgba8unorm-srgb":61,"eac-r11unorm":62,"eac-r11snorm":63,"eac-rg11unorm":64,"eac-rg11snorm":65,"astc-4x4-unorm":66,"astc-4x4-unorm-srgb":67,"astc-5x4-unorm":68,"astc-5x4-unorm-srgb":69,"astc-5x5-unorm":70,"astc-5x5-unorm-srgb":71,"astc-6x5-unorm":72,"astc-6x5-unorm-srgb":73,"astc-6x6-unorm":74,"astc-6x6-unorm-srgb":75,"astc-8x5-unorm":76,"astc-8x5-unorm-srgb":77,"astc-8x6-unorm":78,"astc-8x6-unorm-srgb":79,"astc-8x8-unorm":80,"astc-8x8-unorm-srgb":81,"astc-10x5-unorm":82,"astc-10x5-unorm-srgb":83,"astc-10x6-unorm":84,"astc-10x6-unorm-srgb":85,"astc-10x8-unorm":86,"astc-10x8-unorm-srgb":87,"astc-10x10-unorm":88,"astc-10x10-unorm-srgb":89,"astc-12x10-unorm":90,"astc-12x10-unorm-srgb":91,"astc-12x12-unorm":92,"astc-12x12-unorm-srgb":93,}[ret] ?? 94;
    }, arguments) };
    imports.wbg.__wbg_requestAdapter_b0d64c10f0bfd226 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).requestAdapter(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getBindGroupLayout_d0a82bab7bd9ca14 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_maxTextureDimension1D_ddcb46c74b7a0ecc = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureDimension1D;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxTextureDimension2D_706110d241f13182 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureDimension2D;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxTextureDimension3D_258d3e5dcbb7ae82 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureDimension3D;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxTextureArrayLayers_755cceaa7bf92db3 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureArrayLayers;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxBindGroups_77acf673701b2033 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxBindGroups;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxBindingsPerBindGroup_ee30517e14ff6b3c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxBindingsPerBindGroup;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxDynamicUniformBuffersPerPipelineLayout_5231b2712f207872 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxDynamicStorageBuffersPerPipelineLayout_2162be11827e6a9e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxSampledTexturesPerShaderStage_12c1ad11ed7f078b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxSamplersPerShaderStage_3dd93befa4cc2cfe = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxSamplersPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxStorageBuffersPerShaderStage_212343371d2a6198 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxStorageTexturesPerShaderStage_ccc8044f9154afa1 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxUniformBuffersPerShaderStage_f9e3cbc44c488d8d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxUniformBufferBindingSize_cf56156fa5f6e50f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxUniformBufferBindingSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxStorageBufferBindingSize_f13debb16f988742 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxStorageBufferBindingSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_minUniformBufferOffsetAlignment_f21bc6f52f591b23 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).minUniformBufferOffsetAlignment;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_minStorageBufferOffsetAlignment_9a4902d10ccd3652 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).minStorageBufferOffsetAlignment;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxVertexBuffers_c76a6144b8e6ece0 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxVertexBuffers;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxBufferSize_e54038e4bb003bc8 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxBufferSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxVertexAttributes_84bd4a556f92c239 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxVertexAttributes;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxVertexBufferArrayStride_ecc8a29222dea85e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxVertexBufferArrayStride;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxInterStageShaderComponents_863b889702752696 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxInterStageShaderComponents;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupStorageSize_82448acb20a9efba = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeInvocationsPerWorkgroup_b22189caffbcf407 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupSizeX_b7e9b87440bc44c0 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupSizeX;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupSizeY_fe7fbb52a6a321d5 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupSizeY;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupSizeZ_64d1cfa4663db82f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupsPerDimension_efc3e953c71f81b3 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_gpu_60b5eb17eb957854 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).gpu;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createView_d94df2cf12f51051 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createView(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_destroy_0e473eb42eb825f1 = function() { return logError(function (arg0) {
        getObject(arg0).destroy();
    }, arguments) };
    imports.wbg.__wbg_error_11c623b752f3ff0f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).error;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_queue_15e94b1ed1ba16f8 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).queue;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_Window_ae070805b1226083 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).Window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_WorkerGlobalScope_9af0f1983cb2092e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).WorkerGlobalScope;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestDevice_9c83f27179a99e65 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).requestDevice(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_features_9f72c868d6af0b8d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).features;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_limits_1f1a4bb4c092ca2c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).limits;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_features_0d562755efddf72c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).features;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_limits_55da1c99f0e976e9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).limits;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createShaderModule_f7e713da42dbb7ea = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createShaderModule(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createBindGroupLayout_b0c2f3a6f7d18059 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createBindGroup_5123902bc1e36cc4 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createBindGroup(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createPipelineLayout_37e0e3af31059fc1 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createRenderPipeline_4d68c3e986df2a75 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createComputePipeline_5ae4b1f242668dfa = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createComputePipeline(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createBuffer_8c862fe4a28b2d51 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createBuffer(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createTexture_5f896538314d2e64 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createTexture(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createSampler_de0d16cd11a5cc7b = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createSampler(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createQuerySet_d54619d368d7dd22 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createQuerySet(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createCommandEncoder_9012d7db325fa03e = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createRenderBundleEncoder_5ae7675de454fbf0 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_destroy_8ce1f83528791ca9 = function() { return logError(function (arg0) {
        getObject(arg0).destroy();
    }, arguments) };
    imports.wbg.__wbg_setonuncapturederror_3ee57f1a17c2830d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).onuncapturederror = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_pushErrorScope_ce2f5d4046ca31f6 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).pushErrorScope(["validation","out-of-memory","internal",][arg1]);
    }, arguments) };
    imports.wbg.__wbg_popErrorScope_305d6755b4ec5d8d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).popErrorScope();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_mapAsync_ce27ebba67f67f3f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_unmap_f3d18e3fe556feaf = function() { return logError(function (arg0) {
        getObject(arg0).unmap();
    }, arguments) };
    imports.wbg.__wbg_destroy_405c5391dcc4114d = function() { return logError(function (arg0) {
        getObject(arg0).destroy();
    }, arguments) };
    imports.wbg.__wbg_getBindGroupLayout_2ac2d497e38802ef = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_end_df1f1196b983a2c1 = function() { return logError(function (arg0) {
        getObject(arg0).end();
    }, arguments) };
    imports.wbg.__wbg_end_cf40d9d2d0da0542 = function() { return logError(function (arg0) {
        getObject(arg0).end();
    }, arguments) };
    imports.wbg.__wbg_finish_5753cfe75b8ff1af = function() { return logError(function (arg0) {
        const ret = getObject(arg0).finish();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_finish_b9839222e037a51e = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).finish(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_writeBuffer_a9ad83e7a9ac9d1e = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_usage_91c9f7b31b7b99c9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).usage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_size_ac03167f62f8fc6f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).size;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_writeTexture_e418dedbd3c77a1c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
    }, arguments) };
    imports.wbg.__wbg_copyExternalImageToTexture_48505f2ff1cb0cf0 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_setPipeline_5927afa82a66b006 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setPipeline(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_e6f36794ec41ec5c = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_4740f45d0dd9a40a = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_dispatchWorkgroups_1df593c3d6c79a7c = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_dispatchWorkgroupsIndirect_a8acc06f153b6907 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_setPipeline_e38eb1f97f5ecafa = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setPipeline(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_f94d316567f1d0fc = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_48f3fbe512864ad9 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_204a2b9a6758ab63 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setIndexBuffer(getObject(arg1), ["uint16","uint32",][arg2], arg3);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_67342e26f64e0712 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setIndexBuffer(getObject(arg1), ["uint16","uint32",][arg2], arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_49de4dcb44a2ab41 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_b0f91a955af9a83c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_draw_96226af23cab0d85 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndexed_1c467644a1bc89ff = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndirect_a9bee61f493b639e = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_drawIndexedIndirect_279217c40eb67f68 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_setPipeline_42b1b5a043c178a4 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setPipeline(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_dd735ae90f8f8bb5 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_cf061f92a47bfc35 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_043cf9e6b8d9bab8 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setIndexBuffer(getObject(arg1), ["uint16","uint32",][arg2], arg3);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_06903b407b49be6d = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setIndexBuffer(getObject(arg1), ["uint16","uint32",][arg2], arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_ed7f0780773c2093 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_cfbc801a11b2b2cd = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_draw_9d9deb4ea591ff53 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndexed_650a64a8756c383a = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndirect_867d1f4498970ed6 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_drawIndexedIndirect_468a8501dbcfbef7 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_setBlendConstant_2b8a4d08db94ef4b = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setBlendConstant(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setScissorRect_93f569c9c20465ea = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setViewport_b528e642e8fba393 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
    }, arguments) };
    imports.wbg.__wbg_setStencilReference_851edd0301443d9f = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setStencilReference(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_executeBundles_0af360b832437e34 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).executeBundles(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_submit_f2517011b025285f = function() { return logError(function (arg0, arg1) {
        getObject(arg0).submit(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_instanceof_WebGl2RenderingContext_62ccef896d9204fa = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof WebGL2RenderingContext;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_beginQuery_2babccfce9472da4 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).beginQuery(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindBufferRange_ec55dd1088960c35 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).bindBufferRange(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_bindSampler_f251f0dde3843dc4 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindSampler(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindVertexArray_bec56c40e9ec299d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).bindVertexArray(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_blitFramebuffer_cb1261c0e925d363 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        getObject(arg0).blitFramebuffer(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_bufferData_f552c26392b9837d = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_bufferData_94ce174a81b32961 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_bufferSubData_897bff8bd23ca0b4 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferSubData(arg1 >>> 0, arg2, getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_clearBufferiv_18ffec9d148aaf4b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearBufferiv(arg1 >>> 0, arg2, getArrayI32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_clearBufferuiv_8575fe1b1af9dd15 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearBufferuiv(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_clientWaitSync_8d3b836729fa705f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).clientWaitSync(getObject(arg1), arg2 >>> 0, arg3 >>> 0);
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_compressedTexSubImage2D_d2201c663eb7e7c0 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8, arg9);
    }, arguments) };
    imports.wbg.__wbg_compressedTexSubImage2D_088b90b29f544ebc = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        getObject(arg0).compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, getObject(arg8));
    }, arguments) };
    imports.wbg.__wbg_compressedTexSubImage3D_8d64b364b8ed6808 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        getObject(arg0).compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10, arg11);
    }, arguments) };
    imports.wbg.__wbg_compressedTexSubImage3D_d2b94340686bbb79 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        getObject(arg0).compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, getObject(arg10));
    }, arguments) };
    imports.wbg.__wbg_copyBufferSubData_026e82b392fb8df2 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).copyBufferSubData(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_copyTexSubImage3D_f2471ef3614db8d4 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).copyTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
    }, arguments) };
    imports.wbg.__wbg_createQuery_88b1a8cbfaeadcd4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createQuery();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createSampler_ece1b922a455bd52 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createSampler();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createVertexArray_a3e58c38609ae150 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createVertexArray();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_deleteQuery_deba58de1a061092 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteQuery(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteSampler_341b638a62cece3e = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteSampler(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteSync_ddf848c7dd5cb195 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteSync(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteVertexArray_81346dd52e54eb57 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteVertexArray(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_drawArraysInstanced_c375d32782ea8d30 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_drawBuffers_2744e46ab7e02d91 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).drawBuffers(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_drawElementsInstanced_a416af0d12f00837 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawElementsInstanced(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_endQuery_7e240d815ced0387 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).endQuery(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_fenceSync_0a54247555048537 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).fenceSync(arg1 >>> 0, arg2 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_framebufferTextureLayer_1b5119ac136418d2 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_getBufferSubData_5e2bbbbd18f18d52 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).getBufferSubData(arg1 >>> 0, arg2, getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_getIndexedParameter_edda23e611d65abb = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getIndexedParameter(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getQueryParameter_ec854b270df79577 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getQueryParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getSyncParameter_cf9ca45e037f34f4 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getSyncParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getUniformBlockIndex_8eef3be68190327f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getUniformBlockIndex(getObject(arg1), getStringFromWasm0(arg2, arg3));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_invalidateFramebuffer_12eca43686968fe1 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).invalidateFramebuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_readBuffer_c6e1ba464c45ded1 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).readBuffer(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_readPixels_f589cb77c7641fb2 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        getObject(arg0).readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, getObject(arg7));
    }, arguments) };
    imports.wbg.__wbg_readPixels_74eff76a8a707954 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        getObject(arg0).readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
    }, arguments) };
    imports.wbg.__wbg_renderbufferStorageMultisample_1e0f794803ff8352 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).renderbufferStorageMultisample(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_samplerParameterf_f58c4ac221503b11 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).samplerParameterf(getObject(arg1), arg2 >>> 0, arg3);
    }, arguments) };
    imports.wbg.__wbg_samplerParameteri_97baec154acb369e = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).samplerParameteri(getObject(arg1), arg2 >>> 0, arg3);
    }, arguments) };
    imports.wbg.__wbg_texImage2D_75effcb59fe5da7e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_texImage3D_335fce191a5faae5 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        getObject(arg0).texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, getObject(arg10));
    }, arguments) };
    imports.wbg.__wbg_texStorage2D_6143bf0d71e869ce = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_texStorage3D_5d6b3c6bfa977000 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_be0166513e368886 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_338d11db84a799ed = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_bdc1e6e8b1feae8f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_edb828ed3708cfdd = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_fbb08177c318e3f2 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_c571236e8e9908d5 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_d86e30d5f4ebc0e0 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_b3526f28e3c2031e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_7a0f4d63809a0f6e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
    }, arguments) };
    imports.wbg.__wbg_texSubImage3D_9ee350bf3d5e61ad = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
    }, arguments) };
    imports.wbg.__wbg_uniform1ui_010e62706e661170 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).uniform1ui(getObject(arg1), arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_uniform2fv_83048fbc79c7f362 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform2iv_31ff5561a5c51159 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform2uiv_4b36f1c57b28c3c6 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2uiv(getObject(arg1), getArrayU32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform3fv_0ddd3ca056ab3d1f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform3fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform3iv_eb887b2a339dda97 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform3iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform3uiv_19cbb50d7afeb7d0 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform3uiv(getObject(arg1), getArrayU32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform4fv_cf977e0dd611bbdd = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform4fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform4iv_b3a606d0b1b87dc9 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform4iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform4uiv_cb256e285d564825 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform4uiv(getObject(arg1), getArrayU32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniformBlockBinding_744b2ad6a5f2cace = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniformBlockBinding(getObject(arg1), arg2 >>> 0, arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix2fv_7e757aaedd0427cf = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix2x3fv_91be1a9373d7c5ce = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix2x3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix2x4fv_b5ef5b5baced0e4f = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix2x4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix3fv_5eec5885a8d5de8b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix3x2fv_88709a0858bab333 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix3x2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix3x4fv_184c4f571cff1122 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix3x4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix4fv_ae100fc474463355 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix4x2fv_e931df9c7cb32d55 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix4x2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix4x3fv_f78c83b4908c3e27 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix4x3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_vertexAttribDivisor_48f4c9ce15c07063 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_vertexAttribIPointer_78250ec98da971a2 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).vertexAttribIPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_activeTexture_067b93df6d1ed857 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).activeTexture(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_attachShader_396d529f1d7c9abc = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindAttribLocation_9e7dad25e51f58b1 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).bindAttribLocation(getObject(arg1), arg2 >>> 0, getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_bindBuffer_d6b05e0a99a752d4 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindFramebuffer_f5e959313c29a7c6 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindRenderbuffer_691cb14fc6248155 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindRenderbuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindTexture_840f7fcfd0298dc4 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_blendColor_4c1f00a2e4f1a80d = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).blendColor(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_blendEquation_e7b91e8e062fa502 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).blendEquation(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_blendEquationSeparate_272bfcd932055191 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_blendFunc_6a7b81c06098c023 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_blendFuncSeparate_f81dd232d266e735 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_clear_7a2a7ca897047e8d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clear(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_clearColor_837d30f5bf4f982b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_clearDepth_a65e67fdeb1f3ff9 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearDepth(arg1);
    }, arguments) };
    imports.wbg.__wbg_clearStencil_1f24aec5432f38ba = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearStencil(arg1);
    }, arguments) };
    imports.wbg.__wbg_colorMask_7c2aafdec5441392 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
    }, arguments) };
    imports.wbg.__wbg_compileShader_77ef81728b1c03f6 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).compileShader(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_copyTexSubImage2D_d3b3d3b235c88d33 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        getObject(arg0).copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
    }, arguments) };
    imports.wbg.__wbg_createBuffer_7b18852edffb3ab4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createBuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createFramebuffer_a12847edac092647 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createFramebuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createProgram_73611dc7a72c4ee2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createProgram();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createRenderbuffer_e7bd95fedc0bbcb5 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createRenderbuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createShader_f10ffabbfd8e2c8c = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createTexture_2426b031baa26a82 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createTexture();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_cullFace_fbafcb7763a2d6aa = function() { return logError(function (arg0, arg1) {
        getObject(arg0).cullFace(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_deleteBuffer_27b0fb5ed68afbe4 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteBuffer(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteFramebuffer_c0d511b2fc07620d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteFramebuffer(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteProgram_c3238b647d849334 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteProgram(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteRenderbuffer_325417b497c5ae27 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteRenderbuffer(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteShader_da06706168cf00dc = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteShader(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteTexture_cdd844345a2559bb = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteTexture(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_depthFunc_2f1df7eb8339f5a3 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).depthFunc(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_depthMask_a301dd9951c6056c = function() { return logError(function (arg0, arg1) {
        getObject(arg0).depthMask(arg1 !== 0);
    }, arguments) };
    imports.wbg.__wbg_depthRange_85c249bf5c81856c = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).depthRange(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_disable_8908871f2334e76b = function() { return logError(function (arg0, arg1) {
        getObject(arg0).disable(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_disableVertexAttribArray_79a5010f18eb84cb = function() { return logError(function (arg0, arg1) {
        getObject(arg0).disableVertexAttribArray(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawArrays_7a8f5031b1fe80ff = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
    }, arguments) };
    imports.wbg.__wbg_enable_541ed84c1e7d269d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).enable(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_enableVertexAttribArray_06043f51b716ed9d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_framebufferRenderbuffer_f7c592ad40667f89 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4));
    }, arguments) };
    imports.wbg.__wbg_framebufferTexture2D_5b524fe6135d5fe8 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
    }, arguments) };
    imports.wbg.__wbg_frontFace_54ccf43770ae1011 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).frontFace(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_getExtension_095ef1e6c9d8d8ab = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getExtension(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getParameter_cfaed180705b9280 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).getParameter(arg1 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getProgramInfoLog_fe796f3a9512a8e3 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getProgramParameter_9df6cbbb1343b27d = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getShaderInfoLog_a7ca51b89a4dafab = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getShaderParameter_806970126d526c29 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getSupportedExtensions_e1788ac835b7e81a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getSupportedExtensions();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getUniformLocation_6a59ad54df3bba8e = function() { return logError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_linkProgram_56a5d97f63b1f56d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).linkProgram(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_pixelStorei_3a600280eab03e3c = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
    }, arguments) };
    imports.wbg.__wbg_polygonOffset_ebf1b1bd8db53e65 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).polygonOffset(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_renderbufferStorage_3c5e469d82dfe89b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_scissor_2b172ca4e459dd16 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).scissor(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_shaderSource_b92b2b5c29126344 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_stencilFuncSeparate_25b5dd967d72b6e5 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_stencilMask_702162181d88081f = function() { return logError(function (arg0, arg1) {
        getObject(arg0).stencilMask(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_stencilMaskSeparate_1f803a440e789b81 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_stencilOpSeparate_52b401966f916a0f = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_texParameteri_531d0268109950ba = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    }, arguments) };
    imports.wbg.__wbg_uniform1f_81b570bf6358ae6c = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).uniform1f(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_uniform1i_ded3be13f5d8f11a = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).uniform1i(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_uniform4f_bdbb7cf56fc94cbb = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).uniform4f(getObject(arg1), arg2, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_useProgram_001c6b9208b683d3 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).useProgram(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_vertexAttribPointer_b435a034ff758637 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    }, arguments) };
    imports.wbg.__wbg_viewport_536c78dd69c44351 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).viewport(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Window_5012736c80a01584 = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_document_8554450897a855b9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_navigator_6210380287bf8581 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_devicePixelRatio_7ba8bc80d46340bd = function() { return logError(function (arg0) {
        const ret = getObject(arg0).devicePixelRatio;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_crypto_9d63c6bc9b9f9772 = function() { return handleError(function (arg0) {
        const ret = getObject(arg0).crypto;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_localStorage_90db5cb66e840248 = function() { return handleError(function (arg0) {
        const ret = getObject(arg0).localStorage;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_performance_fa12dc8712926291 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).performance;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_cancelIdleCallback_0d1373370ef859bd = function() { return logError(function (arg0, arg1) {
        getObject(arg0).cancelIdleCallback(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_getComputedStyle_ba4609b39055f674 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).getComputedStyle(getObject(arg1));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_matchMedia_170d35fd154463b2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestIdleCallback_af997f1fdcadcc54 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).requestIdleCallback(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_cancelAnimationFrame_f80ecdad075d1d55 = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).cancelAnimationFrame(arg1);
    }, arguments) };
    imports.wbg.__wbg_requestAnimationFrame_b4b782250b9c2c88 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clearInterval_df3409c32c572e85 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearInterval(arg1);
    }, arguments) };
    imports.wbg.__wbg_clearTimeout_25cdc2ed88b3c0b2 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearTimeout(arg1);
    }, arguments) };
    imports.wbg.__wbg_fetch_f3adf866d8944b41 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).fetch(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setInterval_76a7ba11bc095d2d = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_2bb9dfe810e45e24 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).setTimeout(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_73b734ca971c19f4 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_settitle_64405a7dab688b2b = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).title = getStringFromWasm0(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_body_b3bb488e8e54bf4b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).body;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_visibilityState_51f5bb37c843e94e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).visibilityState;
        return {"hidden":0,"visible":1,}[ret] ?? 2;
    }, arguments) };
    imports.wbg.__wbg_activeElement_1036a8ddc10ec3f1 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).activeElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_fullscreenElement_027a4ad195839d29 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).fullscreenElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createElement_5921e9eb06b9ec89 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_querySelector_e21c39150aa72078 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_querySelectorAll_52447cbab6df8bae = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).querySelectorAll(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setAttribute_d5540a19be09f8dc = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setPointerCapture_16fb4f004fe3aaae = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).setPointerCapture(arg1);
    }, arguments) };
    imports.wbg.__wbg_style_e06c9e03355741e9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).style;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_focus_06621101cc79f5d8 = function() { return handleError(function (arg0) {
        getObject(arg0).focus();
    }, arguments) };
    imports.wbg.__wbg_bufferData_fc33089cf05a6c5a = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_bufferData_0db2a74470353a96 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_bufferSubData_944883045753ee61 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferSubData(arg1 >>> 0, arg2, getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_compressedTexSubImage2D_678be4671393a94b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        getObject(arg0).compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, getObject(arg8));
    }, arguments) };
    imports.wbg.__wbg_readPixels_0c5ad23c72dbe1b8 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        getObject(arg0).readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, getObject(arg7));
    }, arguments) };
    imports.wbg.__wbg_texImage2D_d704e7eee22d1e6b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_texSubImage2D_bed4633ee03b384d = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
        getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
    }, arguments) };
    imports.wbg.__wbg_uniform2fv_b73144e507d90a92 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform2iv_27f3fc3aefa41fa7 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform3fv_5df1d945c0bbfe20 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform3fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform3iv_03be54fcc4468fc4 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform3iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform4fv_d87e4ea9ef6cf6de = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform4fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniform4iv_965df9fa4c8ab47e = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform4iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix2fv_8646addaa18ba00b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix3fv_917f07d03e8b1db5 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix4fv_46c1f9033bbb1a5e = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_activeTexture_b967ed47a8083daa = function() { return logError(function (arg0, arg1) {
        getObject(arg0).activeTexture(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_attachShader_2b5810fc1d23ebe7 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindAttribLocation_0018ec2a523f139f = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).bindAttribLocation(getObject(arg1), arg2 >>> 0, getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_bindBuffer_1f581c747176e7d7 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindFramebuffer_8cba9964befd2a6d = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindRenderbuffer_297ae310683dc32b = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindRenderbuffer(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_bindTexture_bffa89324927e23a = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_blendColor_c876d94aa784bef7 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).blendColor(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_blendEquation_4f3b8eb0b07cab21 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).blendEquation(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_blendEquationSeparate_95241ffd0f6ab09e = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_blendFunc_f31d0f0d227137e0 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_blendFuncSeparate_2b607032f14b9381 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_clear_780c4e5384fe3fc6 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clear(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_clearColor_ac713fa6931cef3c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_clearDepth_92f7c7d02e50df24 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearDepth(arg1);
    }, arguments) };
    imports.wbg.__wbg_clearStencil_78b0b3c82001b542 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearStencil(arg1);
    }, arguments) };
    imports.wbg.__wbg_colorMask_6a64eb75df60e2cf = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
    }, arguments) };
    imports.wbg.__wbg_compileShader_043cc8b99c2efc21 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).compileShader(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_copyTexSubImage2D_8f6644e7df89a307 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
        getObject(arg0).copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
    }, arguments) };
    imports.wbg.__wbg_createBuffer_9571c039ba6696c6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createBuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createFramebuffer_20f79ec189ef2060 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createFramebuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createProgram_2c3a8969b5a76988 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createProgram();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createRenderbuffer_620bdfb7867926e8 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createRenderbuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createShader_af087106532661d9 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createTexture_e49c36c5f31925a3 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createTexture();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_cullFace_ccad99c645b704eb = function() { return logError(function (arg0, arg1) {
        getObject(arg0).cullFace(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_deleteBuffer_898974b9db136e43 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteBuffer(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteFramebuffer_d632dfba2c1f5c75 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteFramebuffer(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteProgram_5f938b0667141206 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteProgram(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteRenderbuffer_ccae7372581ae424 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteRenderbuffer(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteShader_b9bb71cfb1a65a0d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteShader(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_deleteTexture_558c751a66bd2f16 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteTexture(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_depthFunc_5398fbc3f56db827 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).depthFunc(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_depthMask_9b58af067c6393e9 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).depthMask(arg1 !== 0);
    }, arguments) };
    imports.wbg.__wbg_depthRange_29f0e12388f0eacb = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).depthRange(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_disable_d73e59fee5b5e973 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).disable(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_disableVertexAttribArray_b9d8ae826c70526f = function() { return logError(function (arg0, arg1) {
        getObject(arg0).disableVertexAttribArray(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawArrays_532f4e0a4547dd1f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
    }, arguments) };
    imports.wbg.__wbg_enable_68b3fa03a633259a = function() { return logError(function (arg0, arg1) {
        getObject(arg0).enable(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_enableVertexAttribArray_52c23a516be565c0 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_framebufferRenderbuffer_fee6ceb2330389b7 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4));
    }, arguments) };
    imports.wbg.__wbg_framebufferTexture2D_ae81a33228e46de6 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
    }, arguments) };
    imports.wbg.__wbg_frontFace_358bf8c6c5159d54 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).frontFace(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_getParameter_8df84a84197f2148 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).getParameter(arg1 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getProgramInfoLog_22296c36addf7a70 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getProgramParameter_ab2954ca517d8589 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getShaderInfoLog_935361c52a919c15 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getShaderParameter_cedb1ec0d8052eff = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getUniformLocation_9cd213015cf8f29f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_linkProgram_1f18bca817bb6edb = function() { return logError(function (arg0, arg1) {
        getObject(arg0).linkProgram(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_pixelStorei_2498331e094ff305 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
    }, arguments) };
    imports.wbg.__wbg_polygonOffset_6d8d69a8d60e5b82 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).polygonOffset(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_renderbufferStorage_8c3882aa73deada9 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_scissor_d06b14c4966727fa = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).scissor(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_shaderSource_d447b31057e4f64c = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
    }, arguments) };
    imports.wbg.__wbg_stencilFuncSeparate_55376d035e74caf1 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_stencilMask_f55f160fc49b981a = function() { return logError(function (arg0, arg1) {
        getObject(arg0).stencilMask(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_stencilMaskSeparate_578fd1281f54081e = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_stencilOpSeparate_ea6f96abd32aae5b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_texParameteri_83ad7181b62f4997 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    }, arguments) };
    imports.wbg.__wbg_uniform1f_509b4ba100d75456 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).uniform1f(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_uniform1i_7f6e60c975d21e0a = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).uniform1i(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_uniform4f_f9a7809965964840 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).uniform4f(getObject(arg1), arg2, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_useProgram_d4616618ac6d0652 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).useProgram(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_vertexAttribPointer_fcbfe42523d724ca = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    }, arguments) };
    imports.wbg.__wbg_viewport_efc09c09d4f3cc48 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).viewport(arg1, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_setbox_0d838a2d268b7fac = function() { return logError(function (arg0, arg1) {
        getObject(arg0).box = ["border-box","content-box","device-pixel-content-box",][arg1];
    }, arguments) };
    imports.wbg.__wbg_addEventListener_e167f012cbedfa4e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_addEventListener_ad76aa70e65794bd = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
    }, arguments) };
    imports.wbg.__wbg_removeEventListener_b6cef5ad085bea8f = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_isIntersecting_7cba11b732bde6a7 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).isIntersecting;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_altKey_5a6eb49ec8194792 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).altKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_ctrlKey_319ff2374dc7f372 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).ctrlKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shiftKey_f38dee34420e0d62 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).shiftKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_metaKey_00fdcfadf1968d45 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).metaKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_location_d7fe3090ad7e80d7 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).location;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_repeat_8451a79b3608855b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).repeat;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_key_a626396efbca2b95 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).key;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_code_01dc6af887ca9ecb = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).code;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_new_42acb42ec2ace97c = function() { return handleError(function (arg0) {
        const ret = new ResizeObserver(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_disconnect_1dbf7e19d9590abd = function() { return logError(function (arg0) {
        getObject(arg0).disconnect();
    }, arguments) };
    imports.wbg.__wbg_observe_101f5cf5a11e9a79 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).observe(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_observe_60f3631b2f7c6d8b = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).observe(getObject(arg1), getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_unobserve_e67452df241a602d = function() { return logError(function (arg0, arg1) {
        getObject(arg0).unobserve(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setbody_734cb3d7ee8e6e96 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).body = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_setheaders_be10a5ab566fd06f = function() { return logError(function (arg0, arg1) {
        getObject(arg0).headers = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_setmethod_dc68a742c2db5c6a = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).method = getStringFromWasm0(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_getPropertyValue_b0f0858c3b5f17dd = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_removeProperty_cfd836a4f7e5e86e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_setProperty_ff389e5a7fb9910e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_instanceof_HtmlCanvasElement_1a96a01603ec2d8b = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_width_53a5bd0268e99485 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).width;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setwidth_e371a8d6b16ebe84 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_height_6fb32e51e54037ae = function() { return logError(function (arg0) {
        const ret = getObject(arg0).height;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setheight_ba99ad2df4295e89 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_getContext_69ec873410cbba3c = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_70d493702d2b8f3e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2), getObject(arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_persisted_6483200f25cdfed4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).persisted;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_width_151910f38d746773 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).width;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_height_c1b4ecc1cfed30aa = function() { return logError(function (arg0) {
        const ret = getObject(arg0).height;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_pointerId_37ae0c4682f85248 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).pointerId;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_pressure_95cee3909a8549a9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).pressure;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_pointerType_d375491a3013a9bc = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).pointerType;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getCoalescedEvents_bf1c46ef1a01dcce = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCoalescedEvents();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithstrandinit_a31c69e4cc337183 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Response_e91b7eb7c611a9ae = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Response;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_json_3555ed3b0ef0dcad = function() { return handleError(function (arg0) {
        const ret = getObject(arg0).json();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_deltaX_7f4a9de8338c7ca6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).deltaX;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_deltaY_606f12aa66daba69 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).deltaY;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_deltaMode_d6b849e45efd0f5e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).deltaMode;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getprivatekey_38a96140c030aa82 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).privateKey;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getpublickey_8f2e2a04656fbbf4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).publicKey;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_data_5c47a6985fefc490 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).data;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_bindVertexArrayOES_37868a5a4265ea0a = function() { return logError(function (arg0, arg1) {
        getObject(arg0).bindVertexArrayOES(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_createVertexArrayOES_84334a02da216381 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).createVertexArrayOES();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_deleteVertexArrayOES_e22f7a6baedc5300 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).deleteVertexArrayOES(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_width_a7c8cb533b26f0bf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).width;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setwidth_c20f1f8fcd5d93b4 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_height_affa017f56a8fb96 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).height;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setheight_a5e39c9d97429299 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_getContext_bd2ece8a59fd4732 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_76f1b45238db4411 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2), getObject(arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_inlineSize_322ab111c2b5c9e3 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).inlineSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_blockSize_981c4dfa6e1263a8 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).blockSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_code_eae09136895f8ffa = function() { return logError(function (arg0) {
        const ret = getObject(arg0).code;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_width_e7964a50b174d035 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).width;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_height_cd5c897b4d3fabe3 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).height;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_port1_9e11ba1fe63adb21 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).port1;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_port2_7d887905fa4a6677 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).port2;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_40ff2b042829bc58 = function() { return handleError(function () {
        const ret = new MessageChannel();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_framebufferTextureMultiviewOVR_32295d56731dd362 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).framebufferTextureMultiviewOVR(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5, arg6);
    }, arguments) };
    imports.wbg.__wbg_exportKey_fd8bf2738f8734bd = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).exportKey(getStringFromWasm0(arg1, arg2), getObject(arg3));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_generateKey_9433a10876a97b5a = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).generateKey(getObject(arg1), arg2 !== 0, getObject(arg3));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_sign_cbb245aab6e4035f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = getObject(arg0).sign(getObject(arg1), getObject(arg2), getArrayU8FromWasm0(arg3, arg4));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_subtle_26c17db3161dabc0 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).subtle;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getRandomValues_6386cbe348722747 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getRandomValues(getArrayU8FromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getSupportedProfiles_13c2c2008a14070f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getSupportedProfiles();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_error_09480e4aadca50ad = function() { return logError(function (arg0) {
        console.error(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_error_9ce09486992d3ac5 = function() { return logError(function (arg0, arg1) {
        console.error(getObject(arg0), getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_log_b103404cc5920657 = function() { return logError(function (arg0) {
        console.log(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_new_e433ce5049adc572 = function() { return handleError(function (arg0, arg1) {
        const ret = new EventSource(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setonmessage_3b07505d5f934b5c = function() { return logError(function (arg0, arg1) {
        getObject(arg0).onmessage = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_close_87d69f9e9fe928c3 = function() { return logError(function (arg0) {
        getObject(arg0).close();
    }, arguments) };
    imports.wbg.__wbg_postMessage_6eeb375bc13e8ec8 = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).postMessage(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_start_f642a950f6c98a0e = function() { return logError(function (arg0) {
        getObject(arg0).start();
    }, arguments) };
    imports.wbg.__wbg_getItem_cab39762abab3e70 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_readyState_7237e2b1adac03a6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).readyState;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setonclose_40f935717ad6ffcd = function() { return logError(function (arg0, arg1) {
        getObject(arg0).onclose = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_setonmessage_b670c12ea34acd8b = function() { return logError(function (arg0, arg1) {
        getObject(arg0).onmessage = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_setbinaryType_d164a0be4c212c9c = function() { return logError(function (arg0, arg1) {
        getObject(arg0).binaryType = ["blob","arraybuffer",][arg1];
    }, arguments) };
    imports.wbg.__wbg_newwithstrsequence_95750c7542ecfdb3 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = new WebSocket(getStringFromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_close_99bb12a22f16f79c = function() { return handleError(function (arg0) {
        getObject(arg0).close();
    }, arguments) };
    imports.wbg.__wbg_send_1b333b26681a902d = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).send(getArrayU8FromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_signal_41e46ccad44bb5e2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).signal;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_ebf2727385ee825c = function() { return handleError(function () {
        const ret = new AbortController();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_abort_8659d889a7877ae3 = function() { return logError(function (arg0) {
        getObject(arg0).abort();
    }, arguments) };
    imports.wbg.__wbg_drawArraysInstancedANGLE_7c668fc363789760 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).drawArraysInstancedANGLE(arg1 >>> 0, arg2, arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_drawElementsInstancedANGLE_7d0baa058556f76c = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawElementsInstancedANGLE(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_vertexAttribDivisorANGLE_ff0ade84fc10084b = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_preventDefault_c55d86c27b2dfa6e = function() { return logError(function (arg0) {
        getObject(arg0).preventDefault();
    }, arguments) };
    imports.wbg.__wbg_new_d0c6c2df51a6d903 = function() { return handleError(function (arg0) {
        const ret = new IntersectionObserver(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_disconnect_24e89f8d65ad2fd5 = function() { return logError(function (arg0) {
        getObject(arg0).disconnect();
    }, arguments) };
    imports.wbg.__wbg_observe_c901133fbef21560 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).observe(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_appendChild_ac45d1abddf1b89b = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).appendChild(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_contains_4f87c5405416b4fd = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).contains(getObject(arg1));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_get_fe289e3950b3978a = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_navigator_db73b5b11a0c5c93 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_videoWidth_5f4190ae93af0dd6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).videoWidth;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_videoHeight_4fb4bdd27e02263a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).videoHeight;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_media_80f4e313bb8005f1 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).media;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_matches_42eb40a28a316d0e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).matches;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addListener_7d72b16c4161a20a = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).addListener(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_removeListener_0ff79f74b9d808ac = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).removeListener(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_ctrlKey_957c6c31b62b4550 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).ctrlKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shiftKey_8c0f9a5ca3ff8f93 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).shiftKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_altKey_d3fbce7596aac8cf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).altKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_metaKey_be0158b14b1cef4a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).metaKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_button_460cdec9f2512a91 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).button;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_buttons_a302533e27733599 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).buttons;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_movementX_ecd0c638be0899eb = function() { return logError(function (arg0) {
        const ret = getObject(arg0).movementX;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_movementY_3064817f736e8151 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).movementY;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_now_a69647afb1f66247 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).now();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_contentRect_c1a9045c459744d9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).contentRect;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_devicePixelContentBoxSize_8d531ca6a4331b28 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).devicePixelContentBoxSize;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_drawBuffersWEBGL_ff53a7c3360f5716 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).drawBuffersWEBGL(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_log_28eee4e6432efd24 = function() { return logError(function (arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_String_55b8bdc4bc243677 = function() { return logError(function (arg0, arg1) {
        const ret = String(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getElementById_8458f2a6c28467dc = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_settextcontent_fc3ff485b96fcb1d = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).textContent = getStringFromWasm0(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_wbgtestoutputwriteln_4db3bd64914ec955 = function() { return logError(function (arg0) {
        __wbg_test_output_writeln(takeObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_stack_436273c21658169b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).stack;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_static_accessor_document_d4b6ae7f5578480f = function() { return logError(function () {
        const ret = document;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_self_55106357ec10ecd4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).self;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_constructor_fd0d22d60b7dfd72 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).constructor;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_name_7f439d24ff7ba1d3 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).name;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_stack_17c77e9f5bfe6714 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_textcontent_67e4e811cbdf00fc = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).textContent;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_stack_44743fb7d71926a0 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).stack;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_abda76e883ba8a5f = function() { return logError(function () {
        const ret = new Error();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function() { return logError(function (arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).queueMicrotask;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_481971b0d87f3dd4 = function() { return logError(function (arg0) {
        queueMicrotask(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_get_3baa728f9d58d3f6 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_length_ae22078168b726f5 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).length;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_a220cf903aa02ca2 = function() { return logError(function () {
        const ret = new Array();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newnoargs_76313bd6ff35d0f2 = function() { return logError(function (arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_next_de3e9db4440638b2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).next;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_next_f9cb570345655b9a = function() { return handleError(function (arg0) {
        const ret = getObject(arg0).next();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_done_bfda7aa8f252b39f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).done;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_value_6d39332ab4788d86 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).value;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_iterator_888179a48810a9fe = function() { return logError(function () {
        const ret = Symbol.iterator;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_get_224d16597dbbfd96 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_call_1084a111329e68ce = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_525245e2b9901204 = function() { return logError(function () {
        const ret = new Object();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_self_3093d5d1f7bcb682 = function() { return handleError(function () {
        const ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_3bcfc4d31bc012f8 = function() { return handleError(function () {
        const ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_86b222e13bdf32ed = function() { return handleError(function () {
        const ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_e5a3fe56f8be9485 = function() { return handleError(function () {
        const ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_eval_6e4fc17d87772f52 = function() { return handleError(function (arg0, arg1) {
        const ret = eval(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_forEach_1778105a4f7c1a63 = function() { return logError(function (arg0, arg1, arg2) {
        try {
            var state0 = {a: arg1, b: arg2};
            var cb0 = (arg0, arg1, arg2) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_1264(a, state0.b, arg0, arg1, arg2);
                } finally {
                    state0.a = a;
                }
            };
            getObject(arg0).forEach(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    }, arguments) };
    imports.wbg.__wbg_includes_7c12264f911567fe = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).includes(getObject(arg1), arg2);
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_isArray_8364a5371e9737d8 = function() { return logError(function (arg0) {
        const ret = Array.isArray(getObject(arg0));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_of_4a1c869ef05b4b73 = function() { return logError(function (arg0) {
        const ret = Array.of(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_of_99c2a118200b1e62 = function() { return logError(function (arg0, arg1) {
        const ret = Array.of(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_push_37c89022f34c01ca = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).push(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_ArrayBuffer_61dfc3198373c902 = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_message_e18bae0a0e2c097a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).message;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_name_ac78212e803c7941 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).name;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_call_89af060b4e1523f2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_isSafeInteger_7f1ed56200d90674 = function() { return logError(function (arg0) {
        const ret = Number.isSafeInteger(getObject(arg0));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_now_b7a162010a9e75b4 = function() { return logError(function () {
        const ret = Date.now();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_Object_b80213ae6cc9aafb = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_getOwnPropertyDescriptor_104555bb47552c24 = function() { return logError(function (arg0, arg1) {
        const ret = Object.getOwnPropertyDescriptor(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_is_009b1ef508712fda = function() { return logError(function (arg0, arg1) {
        const ret = Object.is(getObject(arg0), getObject(arg1));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_valueOf_d5ba0a54a2aa5615 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).valueOf();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Promise_ae8c7ffdec83f2ae = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Promise;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_b85e72ed1bfd57f9 = function() { return logError(function (arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_1317(a, state0.b, arg0, arg1);
                } finally {
                    state0.a = a;
                }
            };
            const ret = new Promise(cb0);
            return addHeapObject(ret);
        } finally {
            state0.a = state0.b = 0;
        }
    }, arguments) };
    imports.wbg.__wbg_resolve_570458cb99d56a43 = function() { return logError(function (arg0) {
        const ret = Promise.resolve(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_catch_a279b1da46d132d8 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).catch(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_then_95e6edc0f89b73b1 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).then(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_then_876bb3c633745cc6 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_buffer_b7b08af79b0b0974 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_634ada0fd17e2e96 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Int8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_b5293b0eedbac651 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Int16Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_c89d62ca194b7f14 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Int32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_8a2cb9ca96b27ec9 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_ea1883e1e5e86686 = function() { return logError(function (arg0) {
        const ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_set_d1e79e2388520f18 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_length_8339fcf5d8ecd12e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).length;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_bd3d5191e8925067 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Uint16Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_874df3e29cb555f9 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Uint32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_a69c63d7671a5dbf = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Float32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Uint8Array_247a91427532499e = function() { return logError(function (arg0) {
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
    imports.wbg.__wbg_buffer_0710d1b9dbe2eea6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_subarray_7c2e3576afe181d1 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instantiate_81c226291130ef23 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = WebAssembly.instantiate(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Instance_da53c3312320788a = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof WebAssembly.Instance;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_exports_b7a043d9a46df6ae = function() { return logError(function (arg0) {
        const ret = getObject(arg0).exports;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Module_a507a59d32b4cee7 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof WebAssembly.Module;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_Memory_40d721ee89f2c4c2 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof WebAssembly.Memory;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stringify_bbf45426c92a6bf5 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_set_eacc7d73fefaafdf = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
        const v = getObject(arg1);
        const ret = typeof(v) === 'bigint' ? v : undefined;
        if (!isLikeNone(ret)) {
            _assertBigInt(ret);
        }
        getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper662 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_44);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper663 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_47);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper664 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_50);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper665 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_53);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper666 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper667 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper668 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_61);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper669 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_64);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper670 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_67);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper671 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_70);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper672 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper673 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_75);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper674 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_78);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper675 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_81);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper676 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_84);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper677 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_87);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper678 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper679 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_92);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper680 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_95);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper681 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_98);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper682 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_101);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper683 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_104);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper684 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_107);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper685 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_110);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper686 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_113);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper687 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_116);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper689 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_119);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper690 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper691 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_124);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper692 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_127);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper693 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 33, __wbg_adapter_130);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper708 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper719 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 33, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2043 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_137);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2044 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_140);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2045 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_137);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2046 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_137);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2047 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_137);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2048 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_137);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper2049 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 684, __wbg_adapter_137);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper3543 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1504, __wbg_adapter_153);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper3545 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1504, __wbg_adapter_153);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper8758 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 2850, __wbg_adapter_158);
        return addHeapObject(ret);
    }, arguments) };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;



    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined' && Object.getPrototypeOf(module) === Object.prototype)
    ({module} = module)
    else
    console.warn('using deprecated parameters for `initSync()`; pass a single object instead')

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined' && Object.getPrototypeOf(module_or_path) === Object.prototype)
    ({module_or_path} = module_or_path)
    else
    console.warn('using deprecated parameters for the initialization function; pass a single object instead')

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('turbo_genesis_impl_wasm_bindgen_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
