#include <android/log.h>
#include <dlfcn.h>
#include <jni.h>

#include <optional>
#include <string>
#include <string_view>
#include <vector>

#define LOG_TAG "CrypRqJNI"
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#define LOGW(...) __android_log_print(ANDROID_LOG_WARN, LOG_TAG, __VA_ARGS__)
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)

namespace {
struct CrypRqHandleOpaque;

enum CrypRqErrorCode {
    CRYPRQ_OK = 0,
    CRYPRQ_ERR_NULL = 1,
    CRYPRQ_ERR_UTF8 = 2,
    CRYPRQ_ERR_INVALID_ARGUMENT = 3,
    CRYPRQ_ERR_ALREADY_CONNECTED = 4,
    CRYPRQ_ERR_UNSUPPORTED = 5,
    CRYPRQ_ERR_RUNTIME = 6,
    CRYPRQ_ERR_INTERNAL = 255,
};

enum CrypRqConnectionMode {
    CRYPRQ_CONNECTION_MODE_LISTEN = 0,
    CRYPRQ_CONNECTION_MODE_DIAL = 1,
};

struct CrypRqStrView {
    const char *data;
    size_t len;
};

struct CrypRqConfig {
    const char *log_level;
    const CrypRqStrView *allow_peers;
    size_t allow_peers_len;
};

struct CrypRqPeerParams {
    CrypRqConnectionMode mode;
    const char *multiaddr;
};

using cryprq_init_t =
    CrypRqErrorCode (*)(const CrypRqConfig *, CrypRqHandleOpaque **);
using cryprq_connect_t =
    CrypRqErrorCode (*)(CrypRqHandleOpaque *, const CrypRqPeerParams *);
using cryprq_read_packet_t =
    CrypRqErrorCode (*)(CrypRqHandleOpaque *, uint8_t *, size_t, size_t *);
using cryprq_write_packet_t =
    CrypRqErrorCode (*)(CrypRqHandleOpaque *, const uint8_t *, size_t);
using cryprq_on_network_change_t =
    CrypRqErrorCode (*)(CrypRqHandleOpaque *);
using cryprq_close_t = CrypRqErrorCode (*)(CrypRqHandleOpaque *);

void *g_core_handle = nullptr;
cryprq_init_t g_init = nullptr;
cryprq_connect_t g_connect = nullptr;
cryprq_read_packet_t g_read = nullptr;
cryprq_write_packet_t g_write = nullptr;
cryprq_on_network_change_t g_network_change = nullptr;
cryprq_close_t g_close = nullptr;

template <typename T>
T load_symbol(const char *name) {
    if (!g_core_handle) {
        return nullptr;
    }
    void *symbol = dlsym(g_core_handle, name);
    if (!symbol) {
        LOGE("Failed to resolve symbol: %s (%s)", name, dlerror());
        return nullptr;
    }
    return reinterpret_cast<T>(symbol);
}

bool ensure_core_loaded() {
    if (g_core_handle) {
        return true;
    }
    g_core_handle = dlopen("libcryprq_core.so", RTLD_NOW | RTLD_NODELETE);
    if (!g_core_handle) {
        LOGW("dlopen(libcryprq_core.so) failed: %s", dlerror());
        return false;
    }

    g_init = load_symbol<cryprq_init_t>("cryprq_init");
    g_connect = load_symbol<cryprq_connect_t>("cryprq_connect");
    g_read = load_symbol<cryprq_read_packet_t>("cryprq_read_packet");
    g_write = load_symbol<cryprq_write_packet_t>("cryprq_write_packet");
    g_network_change =
        load_symbol<cryprq_on_network_change_t>("cryprq_on_network_change");
    g_close = load_symbol<cryprq_close_t>("cryprq_close");

    if (!g_init || !g_connect || !g_read || !g_write || !g_network_change ||
        !g_close) {
        LOGE("Failed to load required cryprq_core symbols");
        dlclose(g_core_handle);
        g_core_handle = nullptr;
        return false;
    }

    LOGI("cryprq_core loaded successfully");
    return true;
}

std::optional<std::string> jstring_to_string(JNIEnv *env, jstring value) {
    if (!value) {
        return std::nullopt;
    }
    const char *chars = env->GetStringUTFChars(value, nullptr);
    if (!chars) {
        return std::nullopt;
    }
    std::string out(chars);
    env->ReleaseStringUTFChars(value, chars);
    return out;
}

std::vector<std::string> collect_strings(JNIEnv *env, jobjectArray array,
                                         std::vector<CrypRqStrView> *views) {
    std::vector<std::string> storage;
    if (!array) {
        return storage;
    }
    const jsize length = env->GetArrayLength(array);
    storage.reserve(length);
    views->reserve(length);
    for (jsize i = 0; i < length; ++i) {
        auto element =
            static_cast<jstring>(env->GetObjectArrayElement(array, i));
        if (!element) {
            continue;
        }
        auto str = jstring_to_string(env, element);
        env->DeleteLocalRef(element);
        if (!str.has_value()) {
            continue;
        }
        storage.push_back(*str);
        CrypRqStrView view{
            .data = storage.back().c_str(),
            .len = storage.back().size(),
        };
        views->push_back(view);
    }
    return storage;
}

CrypRqHandleOpaque *handle_from_long(jlong handle) {
    return reinterpret_cast<CrypRqHandleOpaque *>(handle);
}

}  // namespace

extern "C" JNIEXPORT jlong JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_nativeInit(
    JNIEnv *env, jclass, jstring logLevel, jobjectArray allowPeers) {
    if (!ensure_core_loaded()) {
        return 0L;
    }

    auto log_level = jstring_to_string(env, logLevel);
    std::vector<CrypRqStrView> views;
    auto allow = collect_strings(env, allowPeers, &views);

    CrypRqConfig config{
        .log_level = log_level ? log_level->c_str() : nullptr,
        .allow_peers = views.empty() ? nullptr : views.data(),
        .allow_peers_len = views.size(),
    };

    CrypRqHandleOpaque *handle = nullptr;
    const auto code = g_init(&config, &handle);
    if (code != CRYPRQ_OK || handle == nullptr) {
        LOGE("cryprq_init failed with code %d", code);
        return 0L;
    }

    return reinterpret_cast<jlong>(handle);
}

extern "C" JNIEXPORT jint JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_nativeConnect(JNIEnv *env, jclass,
                                                      jlong handle, jint mode,
                                                      jstring multiaddr) {
    if (!ensure_core_loaded()) {
        return CRYPRQ_ERR_UNSUPPORTED;
    }

    auto addr = jstring_to_string(env, multiaddr);
    if (!addr.has_value()) {
        return CRYPRQ_ERR_INVALID_ARGUMENT;
    }

    CrypRqPeerParams params{
        .mode = static_cast<CrypRqConnectionMode>(mode),
        .multiaddr = addr->c_str(),
    };

    const auto code = g_connect(handle_from_long(handle), &params);
    return static_cast<jint>(code);
}

extern "C" JNIEXPORT jint JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_nativeReadPacket(JNIEnv *env, jclass,
                                                         jlong handle,
                                                         jbyteArray buffer) {
    if (!ensure_core_loaded()) {
        return CRYPRQ_ERR_UNSUPPORTED;
    }

    jsize len = env->GetArrayLength(buffer);
    jbyte *data = env->GetByteArrayElements(buffer, nullptr);
    size_t out_len = 0;
    auto code = g_read(handle_from_long(handle),
                       reinterpret_cast<uint8_t *>(data),
                       static_cast<size_t>(len), &out_len);
    env->ReleaseByteArrayElements(buffer, data, 0);
    if (code != CRYPRQ_OK) {
        return static_cast<jint>(code);
    }
    return static_cast<jint>(out_len);
}

extern "C" JNIEXPORT jint JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_nativeWritePacket(
    JNIEnv *env, jclass, jlong handle, jbyteArray buffer, jint length) {
    if (!ensure_core_loaded()) {
        return CRYPRQ_ERR_UNSUPPORTED;
    }

    jbyte *data = env->GetByteArrayElements(buffer, nullptr);
    auto code = g_write(handle_from_long(handle),
                        reinterpret_cast<uint8_t *>(data),
                        static_cast<size_t>(length));
    env->ReleaseByteArrayElements(buffer, data, JNI_ABORT);
    return static_cast<jint>(code);
}

extern "C" JNIEXPORT jint JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_nativeOnNetworkChange(JNIEnv *, jclass,
                                                              jlong handle) {
    if (!ensure_core_loaded()) {
        return CRYPRQ_ERR_UNSUPPORTED;
    }
    return static_cast<jint>(g_network_change(handle_from_long(handle)));
}

extern "C" JNIEXPORT void JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_nativeClose(JNIEnv *, jclass,
                                                    jlong handle) {
    if (!ensure_core_loaded()) {
        return;
    }
    if (handle == 0) {
        return;
    }
    g_close(handle_from_long(handle));
}

jint JNI_OnLoad(JavaVM *vm, void *) {
    JNIEnv *env = nullptr;
    if (vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6) != JNI_OK) {
        return JNI_ERR;
    }
    // Defer loading core library until first call.
    return JNI_VERSION_1_6;
}

