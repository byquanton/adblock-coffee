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
java.sourceCompatibility = JavaVersion.VERSION_1_8

publishing {
    publications.create<MavenPublication>("maven") {
        from(components["java"])
    }
}

tasks.register<Exec>("buildRust") {
    workingDir = file("adblock-rs")
    commandLine("./build.sh")
}

tasks.register("copyRustLib", Copy::class) {
    dependsOn("buildRust")
    from("adblock-rs/target/release") {
        include("*.dll", "*.so", "*.dylib")
    }
    into("build/resources/main/native")
}

tasks.named("build") {
    dependsOn("buildRust")
}

tasks.named("processResources") {
    dependsOn("copyRustLib")
}
