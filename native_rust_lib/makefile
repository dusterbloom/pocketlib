# =============================================================================
# Production Build System for iOS and Android Library
# =============================================================================

SHELL := /bin/bash

# Project Configuration
PROJECT_NAME := proofmanager
LIB_NAME := lib$(PROJECT_NAME)

# Environment Detection
ANDROID_HOME ?= $(shell echo $$ANDROID_HOME)
ANDROID_NDK_HOME ?= $(ANDROID_HOME)/ndk/26.3.11579264
ANDROID_NDK_TOOLCHAIN := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64

# Cleaning target directories
CLEAN_TARGETS := $(IOS_BINDINGS_DIR) $(IOS_OUTPUT_DIR) $(IOS_RUST_DIR)


# Output Directories
BINDINGS_DIR := ./bindings
IOS_BINDINGS_DIR := $(BINDINGS_DIR)/ios
ANDROID_BINDINGS_DIR := $(BINDINGS_DIR)/android



ANDROID_OUTPUT_DIR := ../modules/proofmanager/android/src/main

IOS_OUTPUT_DIR := ../modules/proofmanager/ios
IOS_FRAMEWORK_NAME := ProofManager.xcframework
IOS_FRAMEWORK_PATH := $(IOS_OUTPUT_DIR)/$(IOS_FRAMEWORK_NAME) 
IOS_RUST_DIR := $(IOS_OUTPUT_DIR)/rust


# iOS Configuration
IOS_TARGETS := aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin 

# Target directories
IOS_SIM_LIB := target/aarch64-apple-ios-sim/release/$(LIB_NAME).a
IOS_DEVICE_LIB := target/aarch64-apple-ios/release/$(LIB_NAME).a

# Android Configuration
ANDROID_TARGET := aarch64-linux-android
ANDROID_API := 21
ANDROID_ABI_DIR := $(ANDROID_OUTPUT_DIR)/jniLibs/arm64-v8a
ANDROID_KOTLIN_DIR := $(ANDROID_OUTPUT_DIR)/uniffi/proofmanager

# Terminal Colors
BOLD := $(shell tput bold)
GREEN := $(shell tput setaf 2)
YELLOW := $(shell tput setaf 3)
RESET := $(shell tput sgr0)

# Phony Targets
.PHONY: all ios android clean check help ios-build ios-copy android-build android-copy summary

# All target
all: check ios android summary



# Environment Check
check:
	@echo "$(BOLD)Checking build environment...$(RESET)"
	@command -v cargo >/dev/null 2>&1 || { echo "$(BOLD)Error:$(RESET) Rust/Cargo not found. Visit https://rustup.rs/"; exit 1; }
	@command -v xcodebuild >/dev/null 2>&1 || { echo "$(BOLD)Warning:$(RESET) Xcode not found. iOS builds will fail."; }
	@[ -d "$(ANDROID_NDK_HOME)" ] || { echo "$(BOLD)Error:$(RESET) Android NDK not found at $(ANDROID_NDK_HOME)"; exit 1; }
	@echo "$(GREEN)✓ Environment check passed$(RESET)"


# iOS target
ios: ios-build ios-copy

ios-build:
	@echo "$(BOLD)Building for iOS...$(RESET)"
	cargo build
	cargo run --bin uniffi-bindgen generate --library ./target/debug/libproofmanager.dylib --language swift --out-dir ./bindings
	@echo "Building for iOS targets..."
	for target in aarch64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim; do \
		rustup target add $$target; \
		cargo build --release --target=$$target; \
	done

	@echo "Setting up XCFramework..."
	mv ./bindings/proofmanagerFFI.modulemap ./bindings/module.modulemap
	# Clean the output directory first
	rm -rf "ios/ProofManager.xcframework"
	rm -rf "../modules/proofmanager/ios/ProofManager.xcframework"
	xcodebuild -create-xcframework \
		-library ./target/aarch64-apple-ios-sim/release/libproofmanager.a -headers ./bindings \
		-library ./target/aarch64-apple-ios/release/libproofmanager.a -headers ./bindings \
		-output "ios/ProofManager.xcframework"

ios-copy:
	@echo "$(BOLD)Copying iOS artifacts...$(RESET)"
	@mkdir -p $(IOS_OUTPUT_DIR)
	# Copy the XCFramework
	rm -rf ../modules/proofmanager/ios/ProofManager.xcframework
	@cp -r ./ios/ProofManager.xcframework ../modules/proofmanager/ios
	# Copy the Swift bindings
	@cp ./bindings/proofmanager.swift ../modules/proofmanager/ios/
	@echo "$(GREEN)✓ iOS artifacts copied to $(IOS_OUTPUT_DIR)$(RESET)"


# Android target
android: android-build android-copy


android-build:
	@echo "$(BOLD)Building for Android...$(RESET)"
	@echo "→ Adding Android target..."
	@rustup target add $(ANDROID_TARGET) 2>/dev/null || true
	
	@echo "→ Preparing Android environment..."
	@export ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)"
	@export AR="$(ANDROID_NDK_TOOLCHAIN)/bin/llvm-ar"
	@export CC="$(ANDROID_NDK_TOOLCHAIN)/bin/$(ANDROID_TARGET)$(ANDROID_API)-clang"
	@export CXX="$(ANDROID_NDK_TOOLCHAIN)/bin/$(ANDROID_TARGET)$(ANDROID_API)-clang++"
	@export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$(ANDROID_NDK_TOOLCHAIN)/bin/$(ANDROID_TARGET)$(ANDROID_API)-clang"
	
	@echo "→ Building library..."
	@RUSTFLAGS="-C link-arg=-landroid -C link-arg=-llog" \
	cargo build --target $(ANDROID_TARGET) --release 
	
	@echo "→ Generating Kotlin bindings..."
	@mkdir -p $(ANDROID_BINDINGS_DIR)
	@cargo run --features=uniffi/cli --bin uniffi-bindgen generate \
    --library ./target/$(ANDROID_TARGET)/release/$(LIB_NAME).so \
    --language kotlin \
    --out-dir $(ANDROID_BINDINGS_DIR)


android-copy:
	@echo "$(BOLD)Copying Android artifacts...$(RESET)"
	@mkdir -p $(ANDROID_ABI_DIR)
	@mkdir -p $(ANDROID_KOTLIN_DIR)
	@cp ./target/$(ANDROID_TARGET)/release/$(LIB_NAME).so $(ANDROID_ABI_DIR)/
	@if [ -f "$(ANDROID_BINDINGS_DIR)/uniffi/$(PROJECT_NAME)/$(PROJECT_NAME).kt" ]; then \
		cp $(ANDROID_BINDINGS_DIR)/uniffi/$(PROJECT_NAME)/$(PROJECT_NAME).kt $(ANDROID_KOTLIN_DIR)/; \
	else \
		echo "$(YELLOW)Warning: proofmanager.kt not found in $(ANDROID_BINDINGS_DIR)$(RESET)"; \
		exit 1; \
	fi
	@echo "$(GREEN)✓ Android artifacts copied$(RESET)"

# Clean
clean:
	@echo "$(BOLD)Cleaning build artifacts...$(RESET)"
	@cargo clean
	@rm -rf $(CLEAN_TARGETS)
	@rm -rf $(BINDINGS_DIR)
	@rm -rf ./ios/$(IOS_FRAMEWORK_NAME)
	@rm -rf $(IOS_OUTPUT_DIR)/$(IOS_FRAMEWORK_NAME)
	@rm -rf $(ANDROID_ABI_DIR)/$(LIB_NAME).so
	@rm -rf $(ANDROID_KOTLIN_DIR)/$(PROJECT_NAME)*
	@echo "$(GREEN)✓ Clean completed$(RESET)"


# Single source of truth for summary
summary:
	@echo
	@echo "$(BOLD)Build Summary$(RESET)"
	@echo "----------------------------------------"
	@if [ -e "$(IOS_FRAMEWORK_PATH)" ]; then \
		echo "$(BOLD)iOS Outputs:$(RESET)"; \
		echo "  $(GREEN)→$(RESET) Framework:       $(IOS_FRAMEWORK_PATH)"; \
		echo "  $(GREEN)→$(RESET) Swift bindings:  $(IOS_OUTPUT_DIR)/$(PROJECT_NAME).swift"; \
	fi
	@if [ -e "$(ANDROID_ABI_DIR)/$(LIB_NAME).so" ]; then \
		echo "$(BOLD)Android Outputs:$(RESET)"; \
		echo "  $(GREEN)→$(RESET) Library:         $(ANDROID_ABI_DIR)/$(LIB_NAME).so"; \
		echo "  $(GREEN)→$(RESET) Kotlin bindings: $(ANDROID_KOTLIN_DIR)/$(PROJECT_NAME).kt"; \
	fi
	@echo "----------------------------------------"
	@echo "$(GREEN)✓ Build completed successfully!$(RESET)"

# Help
help:
	@echo "$(BOLD)ProofManager Build System$(RESET)"
	@echo ""
	@echo "$(BOLD)Available commands:$(RESET)"
	@echo "  $(GREEN)make$(RESET)               Build everything (iOS and Android)"
	@echo "  $(GREEN)make ios$(RESET)           Build only iOS framework"
	@echo "  $(GREEN)make android$(RESET)       Build only Android library"
	@echo "  $(GREEN)make clean$(RESET)         Clean all build artifacts"
	@echo "  $(GREEN)make check$(RESET)         Verify build environment"
	@echo "  $(GREEN)make help$(RESET)          Show this help message"
	@echo ""
	@echo "$(BOLD)Build outputs:$(RESET)"
	@echo "  iOS: $(IOS_OUTPUT_DIR)/$(IOS_FRAMEWORK_NAME)"
	@echo "  Android: $(ANDROID_ABI_DIR)/$(LIB_NAME).so"