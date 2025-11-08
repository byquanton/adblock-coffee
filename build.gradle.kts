plugins {
    `java-library`
    `maven-publish`
}

repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    testImplementation(libs.junit.junit)
}

group = "eu.byquanton.adblock"
version = "1.2.0"
description = "adblock-coffee"
java.sourceCompatibility = JavaVersion.VERSION_21

publishing {
    publications.create<MavenPublication>("maven") {
        from(components["java"])
    }
}

val rustTargets = mapOf(
    "Linux-x86_64" to "x86_64-unknown-linux-gnu",
    // "linux-aarch64" to "aarch64-unknown-linux-gnu",
    "Windows-x86_64" to "x86_64-pc-windows-gnu",
    // "windows-aarch64" to "aarch64-pc-windows-gnullvm"
)

rustTargets.forEach { (platform, target) ->
    tasks.register<Exec>("cleanRust${platform.replace("-", "_")}") {
        group = "rust"
        description = "Clean Rust build for $platform"
        workingDir = file("adblock-rs")
        commandLine("cargo", "clean", "--target", target)
    }
}

rustTargets.forEach { (platform, target) ->
    val task = tasks.register<Exec>("buildRust${platform.replace("-", "_")}") {
        group = "rust"
        description = "Build Rust library for $platform"
        workingDir = file("adblock-rs")
        environment("RUSTFLAGS", "-Zlocation-detail=none -Zfmt-debug=none")
        commandLine("cargo", "build", "--release", "--target", target)
    }
}

tasks.register("cleanRustAll") {
    group = "rust"
    description = "Clean Rust build for all targets"
    dependsOn(rustTargets.keys.map { "cleanRust${it.replace("-", "_")}" })
}

tasks.register("buildRustAll") {
    group = "rust"
    description = "Build Rust library for all targets"
    dependsOn(rustTargets.keys.map { "buildRust${it.replace("-", "_")}" })
}

tasks.register<Copy>("copyRustLib") {
    dependsOn("buildRustAll")
    rustTargets.forEach { (platform, target) ->
        from("adblock-rs/target/$target/release") {
            include("*.dll", "*.so", "*.dylib")
        }
        into("build/resources/main/native/") // TODO: Naming that doesn't cause conflicts with other arch libs
    }
}

tasks.named("build") {
    dependsOn("buildRustAll")
}

tasks.named("processResources") {
    dependsOn("copyRustLib")
}
