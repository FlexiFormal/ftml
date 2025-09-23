let ftmlViewer;
(function() {
    const __exports = {};
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }
    let wasm = undefined;

    let cachedUint8ArrayMemory0 = null;

    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    let cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

    if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

    function decodeText(ptr, len) {
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    function getStringFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return decodeText(ptr, len);
    }

    const heap = new Array(128).fill(undefined);

    heap.push(undefined, null, true, false);

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    function getObject(idx) { return heap[idx]; }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            wasm.__wbindgen_export_0(addHeapObject(e));
        }
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    let WASM_VECTOR_LEN = 0;

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

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    let cachedDataViewMemory0 = null;

    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

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
        if (builtInMatches && builtInMatches.length > 1) {
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
        : new FinalizationRegistry(
    state => {
        wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b);
    }
    );

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
                    wasm.__wbindgen_export_4.get(state.dtor)(a, state.b);
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
                    wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b); state.a = 0;
                    CLOSURE_DTORS.unregister(state);
                }
            }
        };
        real.original = state;
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    __exports.clear_cache = function() {
        wasm.clear_cache();
    };

    __exports.run = function() {
        wasm.run();
    };

    __exports.print_cache = function() {
        wasm.print_cache();
    };

    function __wbg_adapter_4(arg0, arg1, arg2) {
        wasm.__wbindgen_export_5(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_11(arg0, arg1, arg2) {
        wasm.__wbindgen_export_6(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_14(arg0, arg1, arg2) {
        wasm.__wbindgen_export_7(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_17(arg0, arg1, arg2) {
        wasm.__wbindgen_export_8(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_22(arg0, arg1, arg2) {
        wasm.__wbindgen_export_9(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_27(arg0, arg1) {
        wasm.__wbindgen_export_10(arg0, arg1);
    }

    function __wbg_adapter_30(arg0, arg1) {
        wasm.__wbindgen_export_11(arg0, arg1);
    }

    function __wbg_adapter_33(arg0, arg1, arg2) {
        const ret = wasm.__wbindgen_export_12(arg0, arg1, addHeapObject(arg2));
        return ret >>> 0;
    }

    function __wbg_adapter_447(arg0, arg1, arg2, arg3) {
        wasm.__wbindgen_export_13(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
    }

    /**
     * @enum {0 | 1 | 2 | 3}
     */
    __exports.HighlightStyle = Object.freeze({
        Colored: 0, "0": "Colored",
        Subtle: 1, "1": "Subtle",
        Off: 2, "2": "Off",
        None: 3, "3": "None",
    });

    const __wbindgen_enum_ReadableStreamType = ["bytes"];

    const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];

    const __wbindgen_enum_ScrollLogicalPosition = ["start", "center", "end", "nearest"];

    const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr >>> 0, 1));

    class IntoUnderlyingByteSource {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            IntoUnderlyingByteSourceFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_intounderlyingbytesource_free(ptr, 0);
        }
        /**
         * @returns {number}
         */
        get autoAllocateChunkSize() {
            const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
            return ret >>> 0;
        }
        /**
         * @param {ReadableByteStreamController} controller
         * @returns {Promise<any>}
         */
        pull(controller) {
            const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, addHeapObject(controller));
            return takeObject(ret);
        }
        /**
         * @param {ReadableByteStreamController} controller
         */
        start(controller) {
            wasm.intounderlyingbytesource_start(this.__wbg_ptr, addHeapObject(controller));
        }
        /**
         * @returns {ReadableStreamType}
         */
        get type() {
            const ret = wasm.intounderlyingbytesource_type(this.__wbg_ptr);
            return __wbindgen_enum_ReadableStreamType[ret];
        }
        cancel() {
            const ptr = this.__destroy_into_raw();
            wasm.intounderlyingbytesource_cancel(ptr);
        }
    }
    __exports.IntoUnderlyingByteSource = IntoUnderlyingByteSource;

    const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr >>> 0, 1));

    class IntoUnderlyingSink {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            IntoUnderlyingSinkFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_intounderlyingsink_free(ptr, 0);
        }
        /**
         * @param {any} reason
         * @returns {Promise<any>}
         */
        abort(reason) {
            const ptr = this.__destroy_into_raw();
            const ret = wasm.intounderlyingsink_abort(ptr, addHeapObject(reason));
            return takeObject(ret);
        }
        /**
         * @returns {Promise<any>}
         */
        close() {
            const ptr = this.__destroy_into_raw();
            const ret = wasm.intounderlyingsink_close(ptr);
            return takeObject(ret);
        }
        /**
         * @param {any} chunk
         * @returns {Promise<any>}
         */
        write(chunk) {
            const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, addHeapObject(chunk));
            return takeObject(ret);
        }
    }
    __exports.IntoUnderlyingSink = IntoUnderlyingSink;

    const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr >>> 0, 1));

    class IntoUnderlyingSource {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            IntoUnderlyingSourceFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_intounderlyingsource_free(ptr, 0);
        }
        /**
         * @param {ReadableStreamDefaultController} controller
         * @returns {Promise<any>}
         */
        pull(controller) {
            const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, addHeapObject(controller));
            return takeObject(ret);
        }
        cancel() {
            const ptr = this.__destroy_into_raw();
            wasm.intounderlyingsource_cancel(ptr);
        }
    }
    __exports.IntoUnderlyingSource = IntoUnderlyingSource;

    const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);

                } catch (e) {
                    const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                    if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

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
        imports.wbg.__wbg_Error_1f3748b298f99708 = function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_Number_577a493fc95ea223 = function(arg0) {
            const ret = Number(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_addEventListener_564d5deaecf59624 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) };
        imports.wbg.__wbg_addEventListener_c65b5fc5554e822e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments) };
        imports.wbg.__wbg_add_54585c0c0d5e9d81 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).add(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_add_5a5c5237d27bf98e = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).add(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_altKey_cd080b28d82ba582 = function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        };
        imports.wbg.__wbg_appendChild_024ffa7893da2707 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_body_3af439ac76af2afb = function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_bottom_b76185535c148c2a = function(arg0) {
            const ret = getObject(arg0).bottom;
            return ret;
        };
        imports.wbg.__wbg_buffer_1f897e9f3ed6b41d = function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_byobRequest_ba853121442653bf = function(arg0) {
            const ret = getObject(arg0).byobRequest;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_byteLength_7029fecd0c136e6d = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_byteOffset_8161a341c0d72844 = function(arg0) {
            const ret = getObject(arg0).byteOffset;
            return ret;
        };
        imports.wbg.__wbg_call_2f8d426a20a307fe = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_call_f53f0647ceb9c567 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_cancelAnimationFrame_305d918e5626b4b8 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).cancelAnimationFrame(arg1);
        }, arguments) };
        imports.wbg.__wbg_cancelBubble_ba3496c52eac50f9 = function(arg0) {
            const ret = getObject(arg0).cancelBubble;
            return ret;
        };
        imports.wbg.__wbg_charCodeAt_2b2fda57ce9f1067 = function(arg0, arg1) {
            const ret = getObject(arg0).charCodeAt(arg1 >>> 0);
            return ret;
        };
        imports.wbg.__wbg_checked_973076df2a8a289b = function(arg0) {
            const ret = getObject(arg0).checked;
            return ret;
        };
        imports.wbg.__wbg_childElementCount_bccfea53a39a3cfb = function(arg0) {
            const ret = getObject(arg0).childElementCount;
            return ret;
        };
        imports.wbg.__wbg_childNodes_ef176b83ab95436c = function(arg0) {
            const ret = getObject(arg0).childNodes;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_classList_a0796e6bc192b82b = function(arg0) {
            const ret = getObject(arg0).classList;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_clearTimeout_8d482a1e3e2195df = function(arg0, arg1) {
            getObject(arg0).clearTimeout(arg1);
        };
        imports.wbg.__wbg_clientWidth_e375fa1cadea7de4 = function(arg0) {
            const ret = getObject(arg0).clientWidth;
            return ret;
        };
        imports.wbg.__wbg_clientX_abec4a40877eac64 = function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        };
        imports.wbg.__wbg_clientY_b1d1e636a6fdbb81 = function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        };
        imports.wbg.__wbg_cloneNode_f3cc6cddf7979990 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).cloneNode();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_cloneNode_f87a4a2385ff41a8 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).cloneNode(arg1 !== 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_close_a90439b2444e47b4 = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_close_f602227805f17f95 = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_code_1f6a24376afd1106 = function(arg0, arg1) {
            const ret = getObject(arg1).code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_composedPath_e3274d4b1a25887d = function(arg0) {
            const ret = getObject(arg0).composedPath();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_construct_fa62cd4376ce2d40 = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.construct(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_contains_3eb7d08850418f0e = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
            return ret;
        };
        imports.wbg.__wbg_content_6a23dbed803aa361 = function(arg0) {
            const ret = getObject(arg0).content;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createComment_2ec6c33fc7c11df9 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).createComment(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createElementNS_a9f53206c738878b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createElement_4f7fbf335b949252 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createTextNode_09ae71620b19776d = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createTreeWalker_e362110933c5f055 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).createTreeWalker(getObject(arg1), arg2 >>> 0, getObject(arg3));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_ctrlKey_44c99627901d04ff = function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        };
        imports.wbg.__wbg_deleteProperty_9cff6ac574d03394 = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.deleteProperty(getObject(arg0), getObject(arg1));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_documentElement_2a1000ec3e86fcbd = function(arg0) {
            const ret = getObject(arg0).documentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_document_a6efcd95d74a2ff6 = function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_done_4a7743b6f942c9f3 = function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        };
        imports.wbg.__wbg_enqueue_1e58bed4477a141f = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).enqueue(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_entries_17f7acbc2d691c0d = function(arg0) {
            const ret = Object.entries(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_error_41f0589870426ea4 = function(arg0) {
            console.error(getObject(arg0));
        };
        imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_exec_502f580df7fe533b = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).exec(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_fetch_8de6de50a61a58f2 = function(arg0, arg1) {
            const ret = getObject(arg0).fetch(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_firstChild_318029f00b20eefc = function(arg0) {
            const ret = getObject(arg0).firstChild;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_firstChild_e1285d158f527715 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).firstChild();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_firstElementChild_3de4b13f134b8a9c = function(arg0) {
            const ret = getObject(arg0).firstElementChild;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_focus_bfdfca51cf21f20f = function() { return handleError(function (arg0) {
            getObject(arg0).focus();
        }, arguments) };
        imports.wbg.__wbg_fullscreenElement_fec6d22128b927da = function(arg0) {
            const ret = getObject(arg0).fullscreenElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getAttributeNames_a143e19d53bb6841 = function(arg0) {
            const ret = getObject(arg0).getAttributeNames();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getAttribute_ed701e1b9378e2d6 = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_getBoundingClientRect_ccc70320e28f15c5 = function(arg0) {
            const ret = getObject(arg0).getBoundingClientRect();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getComputedStyle_0ea2f86bd1f3ae01 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).getComputedStyle(getObject(arg1));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getElementById_3d4c5912da7c64a4 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getItem_7083c7e8090b4e20 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getPropertyValue_3b11c8e563b7b775 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getRandomValues_38a1ff1ea09f6cc7 = function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments) };
        imports.wbg.__wbg_getTimezoneOffset_9b0741d5ee85cd60 = function(arg0) {
            const ret = getObject(arg0).getTimezoneOffset();
            return ret;
        };
        imports.wbg.__wbg_get_27b4bcbec57323ca = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_get_5889c00d8fe0f692 = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), arg1 >>> 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_get_59c6316d15f9f1d0 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_get_a045b61469322fcd = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getwithrefkey_1dc361bd10053bfe = function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_hash_d3f2c6df913b8609 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_head_c83aeef13ea35308 = function(arg0) {
            const ret = getObject(arg0).head;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_height_952d49964419974a = function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        };
        imports.wbg.__wbg_host_e3d0174ee47b29da = function(arg0) {
            const ret = getObject(arg0).host;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_id_01c4eccd281f64b9 = function(arg0, arg1) {
            const ret = getObject(arg1).id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_includes_b049685c334daee2 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).includes(getObject(arg1), arg2);
            return ret;
        };
        imports.wbg.__wbg_innerHeight_cf98e7d2da28f860 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).innerHeight;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_innerWidth_67f95b528c9670e7 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).innerWidth;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_insertBefore_ff3cdc8f07aee445 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_instanceof_ArrayBuffer_59339a3a6f0c10ea = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Element_96b17c2bf2f962ff = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Error_1e51a63e1736444c = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_HtmlElement_899b4c5041def2a4 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_HtmlInputElement_6e301f5298c2216e = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_KeyboardEvent_32ffb72b12f636c8 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof KeyboardEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Map_dd89a82d76d1b25f = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Map;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Node_73e55875a825187f = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Node;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_RegExp_f4b1bed10f2769e3 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof RegExp;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Response_0ab386c6818f788a = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_ShadowRoot_ae4bc62016938ece = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ShadowRoot;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Uint8Array_91f3c5adee7e6672 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Window_7f29e5c72acbfd60 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_isArray_55c23353d58297fd = function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_isArray_bc2498eba6fcb71f = function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_isSafeInteger_6091d6e3ee1b65fd = function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_is_a001cd6ada1df292 = function(arg0, arg1) {
            const ret = Object.is(getObject(arg0), getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_iterator_96378c3c9a17347c = function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_json_dbaa85a926a80ed5 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).json();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_keyCode_60d11b4d575c1537 = function(arg0) {
            const ret = getObject(arg0).keyCode;
            return ret;
        };
        imports.wbg.__wbg_key_b316199b492fce18 = function(arg0, arg1) {
            const ret = getObject(arg1).key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_lastChild_0805e91a4bb0c5c1 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).lastChild();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_left_589de3cd45c922d1 = function(arg0) {
            const ret = getObject(arg0).left;
            return ret;
        };
        imports.wbg.__wbg_length_246fa1f85a0dea5b = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_7bca5263dfc69dd1 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_904c0910ed998bf3 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_localStorage_3e7df12e18c45ecc = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).localStorage;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_location_600b8dffb1bdb9c0 = function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_log_0cc1b7768397bcfe = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_log_cb9e190acc5753fb = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_log_f3c04200b995730f = function(arg0) {
            console.log(getObject(arg0));
        };
        imports.wbg.__wbg_mark_7438147ce31e9d4b = function(arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        };
        imports.wbg.__wbg_measure_fb7825c11612c823 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
            }
        }, arguments) };
        imports.wbg.__wbg_message_86bd7dcf158b1dda = function(arg0) {
            const ret = getObject(arg0).message;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_metaKey_902fde49c9a2c28f = function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        };
        imports.wbg.__wbg_name_4eed68a70c7faf2a = function(arg0) {
            const ret = getObject(arg0).name;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_12588505388d0897 = function() { return handleError(function () {
            const ret = new Headers();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_1930cbb8d9ffc31b = function() {
            const ret = new Object();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
            const ret = new Error();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_9190433fb67ed635 = function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_97ddeb994a38bb69 = function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_d5e3800b120e37e1 = function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return __wbg_adapter_447(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return addHeapObject(ret);
            } finally {
                state0.a = state0.b = 0;
            }
        };
        imports.wbg.__wbg_new_eb6fa6c1e9ac9fb7 = function(arg0) {
            const ret = new Date(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newnoargs_a81330f6e05d8aca = function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithargs_17f18261cf81911a = function(arg0, arg1, arg2, arg3) {
            const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_9aade108cd45cf37 = function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithlength_07d24de174e0a5a5 = function(arg0) {
            const ret = new Array(arg0 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithstrandinit_e8e22e9851f3c2fe = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_nextElementSibling_bcd46178649b17ef = function(arg0) {
            const ret = getObject(arg0).nextElementSibling;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_nextNode_e0fc6a18e54ef634 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).nextNode();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_nextSibling_179d0a1601577aec = function(arg0) {
            const ret = getObject(arg0).nextSibling;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_next_2e6b37020ac5fe58 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_next_3de8f2669431a3ff = function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_nodeType_ea36c8ace624a15d = function(arg0) {
            const ret = getObject(arg0).nodeType;
            return ret;
        };
        imports.wbg.__wbg_offsetHeight_99117bc894fe9da5 = function(arg0) {
            const ret = getObject(arg0).offsetHeight;
            return ret;
        };
        imports.wbg.__wbg_offsetWidth_424a14e24715f7ca = function(arg0) {
            const ret = getObject(arg0).offsetWidth;
            return ret;
        };
        imports.wbg.__wbg_outerHTML_7306ec658b1ed630 = function(arg0, arg1) {
            const ret = getObject(arg1).outerHTML;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_ownKeys_cb555391e340ed4d = function() { return handleError(function (arg0) {
            const ret = Reflect.ownKeys(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_parentElement_90704850d3ceb7ee = function(arg0) {
            const ret = getObject(arg0).parentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_parentNode_aa589bbf69854710 = function(arg0) {
            const ret = getObject(arg0).parentNode;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_parseFloat_a818967edadcb1c0 = function(arg0, arg1) {
            const ret = Number.parseFloat(getStringFromWasm0(arg0, arg1));
            return ret;
        };
        imports.wbg.__wbg_prepend_4d3b3f93fd59c2d3 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).prepend(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_preventDefault_9f90a0a7802591fd = function(arg0) {
            getObject(arg0).preventDefault();
        };
        imports.wbg.__wbg_previousNode_10caeccf02f2272c = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).previousNode();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_prototypesetcall_c5f74efd31aea86b = function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        };
        imports.wbg.__wbg_querySelector_cec6e7da2c0386c0 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_querySelector_d9d21debaa778a3a = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_queueMicrotask_bcc6e26d899696db = function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_queueMicrotask_f24a794d09c42640 = function(arg0) {
            queueMicrotask(getObject(arg0));
        };
        imports.wbg.__wbg_readyState_5e428444cc03e863 = function(arg0, arg1) {
            const ret = getObject(arg1).readyState;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_removeAttribute_6930c6c8a4db23d2 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_removeEventListener_93cb6ffe193b71f1 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments) };
        imports.wbg.__wbg_removeEventListener_b25c194da9564efa = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) };
        imports.wbg.__wbg_removeItem_3accb1e04073ade4 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeItem(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_removeProperty_beb176f6c212ad8e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_remove_09af1d8da7bfa7fb = function(arg0) {
            getObject(arg0).remove();
        };
        imports.wbg.__wbg_remove_227e6ca3d1f2d5ff = function(arg0) {
            getObject(arg0).remove();
        };
        imports.wbg.__wbg_remove_502b7d28b0d322ee = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).remove(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_remove_9e7d6044e6345a50 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).remove(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_requestAnimationFrame_d65c9a36afa49236 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_requestFullscreen_510110172de13bff = function() { return handleError(function (arg0) {
            getObject(arg0).requestFullscreen();
        }, arguments) };
        imports.wbg.__wbg_resolve_5775c0ef9222f556 = function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_respond_0003f7c68aa35ef6 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).respond(arg1 >>> 0);
        }, arguments) };
        imports.wbg.__wbg_right_6530f443d3e7f8bb = function(arg0) {
            const ret = getObject(arg0).right;
            return ret;
        };
        imports.wbg.__wbg_root_02c907d4599d5df6 = function(arg0) {
            const ret = getObject(arg0).root;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_scrollIntoView_068dd440e5c264eb = function(arg0, arg1) {
            getObject(arg0).scrollIntoView(arg1 !== 0);
        };
        imports.wbg.__wbg_scrollIntoView_56e8e5add29486e1 = function(arg0, arg1) {
            getObject(arg0).scrollIntoView(getObject(arg1));
        };
        imports.wbg.__wbg_scrollIntoView_b4f9ef4492f7f345 = function(arg0) {
            getObject(arg0).scrollIntoView();
        };
        imports.wbg.__wbg_scrollLeft_57f8d3c367fc3599 = function(arg0) {
            const ret = getObject(arg0).scrollLeft;
            return ret;
        };
        imports.wbg.__wbg_scrollTop_300069f5f35bef4a = function(arg0) {
            const ret = getObject(arg0).scrollTop;
            return ret;
        };
        imports.wbg.__wbg_scrollX_f095c988181b5fec = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).scrollX;
            return ret;
        }, arguments) };
        imports.wbg.__wbg_scrollY_0d170b389c14c0a3 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).scrollY;
            return ret;
        }, arguments) };
        imports.wbg.__wbg_setAttribute_6a3ee9b5deb88ed3 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setItem_328cbcd44fb5d487 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setProperty_850335bbba11775c = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setTimeout_9a6478818dcee76d = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
            return ret;
        }, arguments) };
        imports.wbg.__wbg_set_1d5fe1e3f51a48d8 = function(arg0, arg1, arg2) {
            getObject(arg0).set(getArrayU8FromWasm0(arg1, arg2));
        };
        imports.wbg.__wbg_set_b33e7a98099eed58 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_set_d636a0463acf1dbc = function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        };
        imports.wbg.__wbg_setacceptnode_817ed45249cda178 = function(arg0, arg1) {
            getObject(arg0).acceptNode = getObject(arg1);
        };
        imports.wbg.__wbg_setbehavior_bc641f3c083e5077 = function(arg0, arg1) {
            getObject(arg0).behavior = __wbindgen_enum_ScrollBehavior[arg1];
        };
        imports.wbg.__wbg_setblock_d6efb93ea02be1ff = function(arg0, arg1) {
            getObject(arg0).block = __wbindgen_enum_ScrollLogicalPosition[arg1];
        };
        imports.wbg.__wbg_setcurrentNode_a20cdf88d1fe23ea = function(arg0, arg1) {
            getObject(arg0).currentNode = getObject(arg1);
        };
        imports.wbg.__wbg_setheaders_ac0b1e4890a949cd = function(arg0, arg1) {
            getObject(arg0).headers = getObject(arg1);
        };
        imports.wbg.__wbg_setinnerHTML_fec7cc6bdfe27049 = function(arg0, arg1, arg2) {
            getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setmethod_9ce6e95af1ae0eaf = function(arg0, arg1, arg2) {
            getObject(arg0).method = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setnodeValue_67b8289cc9bf7650 = function(arg0, arg1, arg2) {
            getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setopen_23cb13bf3e64cf1d = function(arg0, arg1) {
            getObject(arg0).open = arg1 !== 0;
        };
        imports.wbg.__wbg_setscrollLeft_f65b744e6f8b23a7 = function(arg0, arg1) {
            getObject(arg0).scrollLeft = arg1;
        };
        imports.wbg.__wbg_setscrollTop_3e7a7b93db3dbc6b = function(arg0, arg1) {
            getObject(arg0).scrollTop = arg1;
        };
        imports.wbg.__wbg_settextContent_8dc3dd5f9e65733c = function(arg0, arg1, arg2) {
            getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setvalue_5be202fbc5b373d4 = function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setx_f4758dc62d555b81 = function(arg0, arg1) {
            getObject(arg0).x = arg1;
        };
        imports.wbg.__wbg_sety_ac4b437f4d470f13 = function(arg0, arg1) {
            getObject(arg0).y = arg1;
        };
        imports.wbg.__wbg_slice_806fc0a0f411a5b5 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).slice(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_1f13249cc3acc96d = function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_THIS_df7ae94b1e0ed6a3 = function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_SELF_6265471db3b3c228 = function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_WINDOW_16fb482f8ec52863 = function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_status_31874648c8651949 = function(arg0) {
            const ret = getObject(arg0).status;
            return ret;
        };
        imports.wbg.__wbg_stopPropagation_bbcd5bfce38f8942 = function(arg0) {
            getObject(arg0).stopPropagation();
        };
        imports.wbg.__wbg_stringify_1f41b6198e0932e0 = function() { return handleError(function (arg0) {
            const ret = JSON.stringify(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_stringify_a9ae7263cd1aab65 = function(arg0, arg1) {
            const ret = JSON.stringify(getObject(arg1));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_style_f3f621cf77281571 = function(arg0) {
            const ret = getObject(arg0).style;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_tagName_daba242cc67be72b = function(arg0, arg1) {
            const ret = getObject(arg1).tagName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_target_bfb4281bfa013115 = function(arg0) {
            const ret = getObject(arg0).target;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_textContent_094fcf277dd9df42 = function(arg0, arg1) {
            const ret = getObject(arg1).textContent;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_text_42c080764c927da6 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).text();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_then_8d2fcccde5380a03 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_9cc266be2bf537b6 = function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_toString_1588a16751ba3f70 = function(arg0) {
            const ret = getObject(arg0).toString();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_top_745049fe1e355a3f = function(arg0) {
            const ret = getObject(arg0).top;
            return ret;
        };
        imports.wbg.__wbg_value_09d0b4eaab48b91d = function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_value_b3bb6dd468d1cb71 = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_value_d042b03e4d34b965 = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_view_d36d28552eb70661 = function(arg0) {
            const ret = getObject(arg0).view;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_wbindgenbigintgetasi64_7637cb1a7fb9a81e = function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbg_wbindgenbooleanget_59f830b1a70d2530 = function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        };
        imports.wbg.__wbg_wbindgencbdrop_a85ed476c6a370b9 = function(arg0) {
            const obj = getObject(arg0).original;
            if (obj.cnt-- == 1) {
                obj.a = 0;
                return true;
            }
            const ret = false;
            return ret;
        };
        imports.wbg.__wbg_wbindgendebugstring_bb652b1bc2061b6d = function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_wbindgenin_192b210aa1c401e9 = function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_wbindgenisbigint_7d76a1ca6454e439 = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        };
        imports.wbg.__wbg_wbindgenisfalsy_a15841cca6156044 = function(arg0) {
            const ret = !getObject(arg0);
            return ret;
        };
        imports.wbg.__wbg_wbindgenisfunction_ea72b9d66a0e1705 = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        };
        imports.wbg.__wbg_wbindgenisnull_e1388bbe88158c3f = function(arg0) {
            const ret = getObject(arg0) === null;
            return ret;
        };
        imports.wbg.__wbg_wbindgenisobject_dfe064a121d87553 = function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        };
        imports.wbg.__wbg_wbindgenisstring_4b74e4111ba029e6 = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        };
        imports.wbg.__wbg_wbindgenisundefined_71f08a6ade4354e7 = function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        };
        imports.wbg.__wbg_wbindgenjsvaleq_f27272c0a890df7f = function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_wbindgenjsvallooseeq_9dd7bb4b95ac195c = function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_wbindgennumberget_d855f947247a3fbc = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbg_wbindgenstringget_43fe05afe34b0cb1 = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_wbindgenthrow_4c11a24fca429ccf = function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        };
        imports.wbg.__wbg_wbindgentryintonumber_c9e75d941453b6d7 = function(arg0) {
            let result;
            try { result = +getObject(arg0) } catch (e) { result = e }
            const ret = result;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_width_4bc08bbbaf10af06 = function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        };
        imports.wbg.__wbg_x_64cb554cc7eb4d03 = function(arg0) {
            const ret = getObject(arg0).x;
            return ret;
        };
        imports.wbg.__wbg_x_73cfdc021caa2561 = function(arg0) {
            const ret = getObject(arg0).x;
            return ret;
        };
        imports.wbg.__wbg_y_cba0cb8e0b18f007 = function(arg0) {
            const ret = getObject(arg0).y;
            return ret;
        };
        imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_37629468bded5eb4 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 7058, function: Function { arguments: [NamedExternref("Event")], shim_idx: 7085, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 7058, __wbg_adapter_14);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_40c510fdedff2912 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6907, function: Function { arguments: [], shim_idx: 6910, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6907, __wbg_adapter_27);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_4625c577ab2ec9ee = function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_926420ee0c28abd7 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 7059, function: Function { arguments: [], shim_idx: 7084, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 7059, __wbg_adapter_30);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_9ae0607507abb057 = function(arg0) {
            // Cast intrinsic for `I64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_9f7e054add35e530 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 7122, function: Function { arguments: [Externref], shim_idx: 7133, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 7122, __wbg_adapter_22);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_a2e47d73655b2aa6 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 4795, function: Function { arguments: [NamedExternref("Event")], shim_idx: 4932, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, 4795, __wbg_adapter_17);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_b9b8460728623159 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 5599, function: Function { arguments: [NamedExternref("Node")], shim_idx: 5975, ret: U32, inner_ret: Some(U32) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, 5599, __wbg_adapter_33);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_c89858e7a0ea4b60 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6907, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6911, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6907, __wbg_adapter_11);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_eacbfb644e0fb21e = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6763, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 6764, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6763, __wbg_adapter_4);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
            takeObject(arg0);
        };

        return imports;
    }

    function __wbg_init_memory(imports, memory) {

    }

    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        __wbg_init.__wbindgen_wasm_module = module;
        cachedDataViewMemory0 = null;
        cachedUint8ArrayMemory0 = null;


        wasm.__wbindgen_start();
        return wasm;
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (typeof module !== 'undefined') {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

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


        if (typeof module_or_path !== 'undefined') {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (typeof module_or_path === 'undefined' && typeof script_src !== 'undefined') {
            module_or_path = script_src.replace(/\.js$/, '_bg.wasm');
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        __wbg_init_memory(imports);

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    ftmlViewer = Object.assign(__wbg_init, { initSync }, __exports);

})();
ftmlViewer();
