// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "BopixiCore",
    /* CryptoKit (AppleSignIn) needs 10.15 when the checks run on macOS */
    platforms: [.macOS(.v10_15), .iOS(.v16)],
    products: [.library(name: "BopixiCore", targets: ["BopixiCore"])],
    targets: [
        .target(name: "BopixiCore", path: "Bopixi/Core"),
        .executableTarget(name: "BopixiCoreChecks", dependencies: ["BopixiCore"], path: "Tests")
    ]
)
