// Maud Extensions Signals Adapter
(function () {
    const core = window.preactSignalsCore;
    if (!core) {
        console.warn("maud-extensions: signals adapter loaded before @preact/signals-core");
        return;
    }

    const mx = window.mx ??= {};
    if (mx.__signalsAdapterInstalled) {
        return;
    }
    mx.__signalsAdapterInstalled = true;

    const SIGNAL_BRAND = Symbol.for("preact-signals");
    const COMPONENT_SELECTOR = "[data-mx-component]";
    const cleanupByRoot = window.mxCleanupByRoot ??= new Map();

    function warn(message) {
        console.warn(`maud-extensions: ${message}`);
    }

    function isElement(value) {
        return typeof Element !== "undefined" && value instanceof Element;
    }

    function isCollection(value) {
        return (
            (typeof NodeList !== "undefined" && value instanceof NodeList) ||
            Array.isArray(value)
        );
    }

    function isSignalLike(value) {
        return (
            typeof value === "object" &&
            value !== null &&
            value.brand === SIGNAL_BRAND &&
            "value" in value
        );
    }

    function safeDispose(disposer) {
        if (typeof disposer === "function") {
            disposer();
        } else if (disposer && typeof disposer[Symbol.dispose] === "function") {
            disposer[Symbol.dispose]();
        }
    }

    function ensureCleanupScope(root) {
        let scope = cleanupByRoot.get(root);
        if (!scope) {
            scope = { disposers: new Set(), disposed: false };
            cleanupByRoot.set(root, scope);
        }
        return scope;
    }

    function registerCleanup(root, disposer) {
        const scope = ensureCleanupScope(root);
        if (!scope || scope.disposed) {
            safeDispose(disposer);
            return false;
        }
        scope.disposers.add(disposer);
        return true;
    }

    function cleanupRoot(root) {
        const scope = cleanupByRoot.get(root);
        if (!scope || scope.disposed) {
            return;
        }
        scope.disposed = true;
        scope.disposers.forEach((disposer) => {
            try {
                safeDispose(disposer);
            } catch (error) {
                console.warn("maud-extensions: cleanup disposer failed", error);
            }
        });
        scope.disposers.clear();
        cleanupByRoot.delete(root);
    }

    function cleanupRemovedNode(node) {
        if (!isElement(node)) {
            return;
        }
        if (typeof node.matches === "function" && node.matches(COMPONENT_SELECTOR)) {
            cleanupRoot(node);
        }
        if (typeof node.querySelectorAll === "function") {
            node.querySelectorAll(COMPONENT_SELECTOR).forEach((root) => {
                cleanupRoot(root);
            });
        }
    }

    function startCleanupObserver() {
        if (window.mxCleanupObserver || typeof MutationObserver === "undefined") {
            return;
        }
        if (typeof document === "undefined" || !document.documentElement) {
            return;
        }
        window.mxCleanupObserver = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                mutation.removedNodes.forEach((node) => {
                    cleanupRemovedNode(node);
                });
            });
        });
        window.mxCleanupObserver.observe(document.documentElement, {
            childList: true,
            subtree: true,
        });
    }

    function resolveComponentRoot(element) {
        if (!isElement(element) || typeof element.closest !== "function") {
            return null;
        }
        return element.closest(COMPONENT_SELECTOR);
    }

    function validateSource(source, binderName) {
        if (typeof source === "function" || isSignalLike(source)) {
            return true;
        }
        warn(`${binderName}() expects a signal or function source`);
        return false;
    }

    function readSourceValue(source) {
        const value = typeof source === "function" ? source() : source;
        return isSignalLike(value) ? value.value : value;
    }

    function toElements(target, binderName) {
        if (isElement(target)) {
            return [target];
        }
        if (isCollection(target)) {
            return Array.from(target).filter(isElement);
        }
        warn(`${binderName}() expects an element, NodeList, or array of elements`);
        return null;
    }

    function bindEffect(target, source, binderName, applyValue) {
        if (!validateSource(source, binderName)) {
            return target;
        }

        const elements = toElements(target, binderName);
        if (!elements) {
            return target;
        }

        elements.forEach((element) => {
            const root = resolveComponentRoot(element);
            if (!root) {
                warn(`${binderName}() requires a target inside a component! root`);
                return;
            }

            startCleanupObserver();

            const dispose = core.effect(() => {
                applyValue(element, readSourceValue(source));
            });
            registerCleanup(root, dispose);
        });

        return target;
    }

    function validateName(name, binderName, kind) {
        if (typeof name === "string" && name.length > 0) {
            return true;
        }
        warn(`${binderName}() requires a non-empty ${kind} name`);
        return false;
    }

    function validateClassName(name, binderName) {
        if (!validateName(name, binderName, "class")) {
            return false;
        }
        if (/\s/.test(name)) {
            warn(`${binderName}() requires a single class token`);
            return false;
        }
        return true;
    }

    function attachElementBindings(element) {
        if (!isElement(element)) {
            return element;
        }

        element.bindText = (source) => mx.bindText(element, source);
        element.bind_text = element.bindText;
        element.bindAttr = (name, source) => mx.bindAttr(element, name, source);
        element.bind_attr = element.bindAttr;
        element.bindClass = (name, source) => mx.bindClass(element, name, source);
        element.bind_class = element.bindClass;
        element.bindShow = (source) => mx.bindShow(element, source);
        element.bind_show = element.bindShow;
        return element;
    }

    function attachCollectionBindings(collection) {
        if (!isCollection(collection)) {
            return collection;
        }

        collection.bindText = (source) => mx.bindText(collection, source);
        collection.bind_text = collection.bindText;
        collection.bindAttr = (name, source) => mx.bindAttr(collection, name, source);
        collection.bind_attr = collection.bindAttr;
        collection.bindClass = (name, source) => mx.bindClass(collection, name, source);
        collection.bind_class = collection.bindClass;
        collection.bindShow = (source) => mx.bindShow(collection, source);
        collection.bind_show = collection.bindShow;
        return collection;
    }

    function attachBindings(target) {
        if (isCollection(target)) {
            Array.from(target).forEach(attachElementBindings);
            return attachCollectionBindings(target);
        }
        return attachElementBindings(target);
    }

    function wrapSurrealSelector(fn) {
        if (typeof fn !== "function") {
            return null;
        }

        const wrapped = function (...args) {
            return attachBindings(fn.apply(this, args));
        };
        wrapped.__maudExtensionsSignalsAdapter = true;
        return wrapped;
    }

    mx.signal = core.signal;
    mx.computed = core.computed;
    mx.effect = core.effect;
    mx.batch = core.batch;
    mx.untracked = core.untracked;

    mx.bindText = function bindText(target, source) {
        return bindEffect(target, source, "bindText", (element, value) => {
            element.textContent = value == null ? "" : String(value);
        });
    };

    mx.bindAttr = function bindAttr(target, name, source) {
        if (!validateName(name, "bindAttr", "attribute")) {
            return target;
        }
        return bindEffect(target, source, "bindAttr", (element, value) => {
            if (value == null || value === false) {
                element.removeAttribute(name);
            } else if (value === true) {
                element.setAttribute(name, "");
            } else {
                element.setAttribute(name, String(value));
            }
        });
    };

    mx.bindClass = function bindClass(target, className, source) {
        if (!validateClassName(className, "bindClass")) {
            return target;
        }
        return bindEffect(target, source, "bindClass", (element, value) => {
            element.classList.toggle(className, Boolean(value));
        });
    };

    mx.bindShow = function bindShow(target, source) {
        return bindEffect(target, source, "bindShow", (element, value) => {
            element.hidden = !Boolean(value);
        });
    };

    const wrappedMe = wrapSurrealSelector(window.me);
    if (wrappedMe) {
        window.me = wrappedMe;
        if (window.document) {
            window.document.me = wrappedMe;
        }
    }

    const wrappedAny = wrapSurrealSelector(window.any);
    if (wrappedAny) {
        window.any = wrappedAny;
        if (window.document) {
            window.document.any = wrappedAny;
        }
    }
})();
