import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import java.util.Properties

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.ksp)
}

android {
    namespace = "br.acerola.comic.rust"
    compileSdk = 36
    defaultConfig {
        minSdk = 26
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
    packaging {
        jniLibs {
            pickFirsts.add("**/libjnidispatch.so")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }
    buildFeatures {
        compose = false
        buildConfig = true
    }
    kotlin {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_21)
        }
    }
    testOptions {
        unitTests.isReturnDefaultValues = true
    }
}

kotlin {
    compilerOptions {
        freeCompilerArgs.add("-Xsuppress-version-warnings")
    }
}

dependencies {
    testImplementation(libs.mockk)
    testImplementation(libs.junit)
    // Testes instrumentados que atravessam a FFI/JNA de verdade (binário .so real
    // compilado pro dispositivo, não um mock Rust puro) — ver
    // src/androidTest/.../FileSyncFfiIntegrationTest.kt.
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.test.core.ktx)
    // `androidx.test.ext:junit` não traz o `AndroidJUnitRunner` em si — sem isso o
    // `testInstrumentationRunner` declarado acima falha com ClassNotFoundException.
    androidTestImplementation(libs.androidx.test.runner)
    implementation(libs.androidx.annotation)
    implementation(libs.kotlinx.coroutines.core)
    implementation(variantOf(libs.jna) { artifactType("aar") })
}

configure<org.jlleitschuh.gradle.ktlint.KtlintExtension> {
    filter {
        exclude { it.file.path.contains("p2p") }
        exclude { it.file.path.contains("generated") }
    }
}

tasks.withType<Test>().configureEach {
    // Garante que gere a lib e os bindings antes de rodar os testes unitários
    dependsOn("generateBindings", "copyRustLib")

    systemProperty("java.library.path", "${project.projectDir}/src/main/jniLibs/arm64-v8a")
    systemProperty("jna.library.path", "${project.projectDir}/src/main/jniLibs/arm64-v8a")

    testLogging {
        events("passed", "skipped", "failed")
    }
}

val localProperties: Properties by lazy {
    val props = Properties()
    val localPropsFile = rootProject.file("local.properties")
    if (localPropsFile.exists()) {
        localPropsFile.inputStream().use { props.load(it) }
    }
    props
}

val cargo: String =
    project.findProperty("cargo.dir") as? String
        ?: System.getenv("CARGO")
        ?: localProperties.getProperty("cargo.dir")
        ?: "cargo"

// O build Rust/UniFFI inteiro (compilar, gerar bindings, copiar .so) é delegado ao
// `cargo make` (ver `rust/Makefile.toml`) em vez de várias tasks `Exec` chamando `cargo`
// direto — a lógica de cross-platform copy/mkdir fica no duckscript do cargo-make em vez
// de `ifeq ($(OS),Windows_NT)` espalhado, e adicionar uma nova variante de build (ex.:
// x86_64 pra emulador) é só uma task nova no Makefile.toml, não mexe no Gradle.
val cargoMake: String =
    project.findProperty("cargo.make.dir") as? String
        ?: System.getenv("CARGO_MAKE")
        ?: localProperties.getProperty("cargo.make.dir")
        ?: "cargo-make"

// Valida que o `cargo-make` resolvido existe ANTES de qualquer task tentar usá-lo — sem
// isso, a falha que aparece é um erro cru de processo nativo não encontrado, sem dizer
// que era o cargo-make que faltava nem como configurar um caminho customizado.
val validateCargoMake by
    tasks.registering {
        doFirst {
            // `cargo-make.exe` espera receber `make` como primeiro argumento (é assim que o
            // `cargo` internamente invoca qualquer plugin `cargo-XXX`) — chamar só
            // `cargo-make --version` direto quebra com um erro de parser confuso.
            val found =
                runCatching {
                    ProcessBuilder(cargoMake, "make", "--version").redirectErrorStream(true).start().waitFor() == 0
                }.getOrDefault(false)

            if (!found) {
                throw GradleException(
                    "cargo-make não encontrado (procurado em '$cargoMake'). Instale com " +
                        "`cargo install cargo-make`, ou configure o caminho via a propriedade de " +
                        "projeto `cargo.make.dir`, a env var CARGO_MAKE, ou `cargo.make.dir` em " +
                        "local.properties.",
                )
            }
        }
    }

tasks.register<Exec>("buildRust") {
    dependsOn(validateCargoMake)
    workingDir = file("rust")
    commandLine(cargoMake, "make", "build-rust")
}

tasks.register<Exec>("generateBindings") {
    dependsOn("buildRust")
    workingDir = file("rust")
    commandLine(cargoMake, "make", "generate-bindings")
}

tasks.register<Exec>("copyRustLib") {
    dependsOn("buildRust")
    workingDir = file("rust")
    commandLine(cargoMake, "make", "copy-libs")
}

// Não entra no `preBuild`/`connectedAndroidTest` por padrão — só é necessária pra rodar
// `connectedAndroidTest` num EMULADOR (x86_64), que não carrega o .so arm64-v8a padrão.
// Rode manualmente antes: `./gradlew :native:buildRustEmulator`.
tasks.register<Exec>("buildRustEmulator") {
    dependsOn(validateCargoMake)
    workingDir = file("rust")
    commandLine(cargoMake, "make", "build-emulator")
}

tasks.named("preBuild") {
    dependsOn("generateBindings", "copyRustLib")
}

tasks.configureEach {
    val isJniTask = name.startsWith("merge") && name.contains("JniLib")
    val isTestTask = name.contains("AndroidTest") || name.startsWith("connected")

    if (isJniTask || isTestTask) {
        dependsOn("generateBindings", "copyRustLib")
    }
}
