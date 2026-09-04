#[path = "lib/mode.rs"]
pub(crate) mod mode;

#[path = "lib/singleton.rs"]
pub(crate) mod singleton;

#[path = "lib/sync_direction.rs"]
pub(crate) mod sync_direction;

pub(crate) mod callbacks;
pub(crate) mod fsutil;
pub(crate) mod protocol;
pub(crate) mod storage;
pub(crate) mod trust_store;

pub mod api;

uniffi::setup_scaffolding!();

#[cfg(target_os = "android")]
mod android_init {
    use jni::objects::GlobalRef;
    use once_cell::sync::OnceCell;
    use tracing_subscriber::filter::{LevelFilter, Targets};
    use tracing_subscriber::prelude::*;

    static CONTEXT: OnceCell<GlobalRef> = OnceCell::new();

    #[allow(non_snake_case)]
    #[no_mangle]
    pub extern "system" fn JNI_OnLoad(vm: jni::JavaVM, _: *mut std::ffi::c_void) -> jni::sys::jint {
        let mut env = vm
            .attach_current_thread()
            .expect("Failed to attach JVM thread");

        let activity_thread = env
            .find_class("android/app/ActivityThread")
            .expect("ActivityThread not found");

        let context = env
            .call_static_method(
                activity_thread,
                "currentApplication",
                "()Landroid/app/Application;",
                &[],
            )
            .expect("currentApplication failed")
            .l()
            .expect("Expected object");

        let global = env
            .new_global_ref(context)
            .expect("Failed to create global ref");

        unsafe {
            ndk_context::initialize_android_context(
                vm.get_java_vm_pointer().cast(),
                global.as_obj().as_raw().cast(),
            );
        }

        CONTEXT.set(global).ok();

        tracing_subscriber::registry()
            .with(tracing_android::layer("ACEROLA/P2P").unwrap())
            .with(
                Targets::new()
                    .with_default(LevelFilter::WARN)
                    .with_target("acerola_p2p", LevelFilter::DEBUG)
                    .with_target("acerola", LevelFilter::DEBUG)
                    .with_target("acto", LevelFilter::OFF)
                    .with_target("iroh", LevelFilter::DEBUG) // temporário
                    .with_target("noq_proto", LevelFilter::DEBUG) // temporário
                    .with_target("swarm_discovery", LevelFilter::OFF)
                    .with_target("hickory_proto", LevelFilter::OFF)
                    .with_target("iroh", LevelFilter::ERROR),
            )
            .init();

        jni::sys::JNI_VERSION_1_6
    }
}
