# =============================================================================
# Improved Build System for iOS and Android Library
# =============================================================================

SHELL := /bin/bash

# Project Configuration
PROJECT_NAME := proofmanager
LIB_NAME := lib$(PROJECT_NAME)

# Environment Detection
ANDROID_HOME ?= $(shell echo $$ANDROID_HOME)
ANDROID_NDK_HOME ?= $(ANDROID_HOME)/ndk/26.3.11579264
ANDROID_NDK_TOOLCHAIN := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64

# Android Configuration
ANDROID_TARGET := aarch64-linux-android
ANDROID_API := 21

# Source locations
MODULE_SOURCE := ../modules/proofmanager/android/src/main/java/expo/modules/proofmanager/ProofManagerModule.kt

ANDROID_OUTPUT_DIR := ../android/app/src/main
ANDROID_ABI_DIR := $(ANDROID_OUTPUT_DIR)/jniLibs/arm64-v8a
ANDROID_KOTLIN_DIR := $(ANDROID_OUTPUT_DIR)/uniffi/proofmanager
ANDROID_MODULE_DIR := $(ANDROID_OUTPUT_DIR)/java/expo/modules/proofmanager

# iOS Configuration
IOS_TARGETS := aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin 
IOS_OUTPUT_DIR := ../modules/proofmanager/ios
IOS_FRAMEWORK_NAME := ProofManager.xcframework
IOS_FRAMEWORK_PATH := $(IOS_OUTPUT_DIR)/$(IOS_FRAMEWORK_NAME)

# Bindings Directories
BINDINGS_DIR := ./bindings

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

# iOS Build and Copy
# iOS target
ios: ios-build ios-copy 

# Build target - handles compilation of Rust code
ios-build:
	@echo "Building library and generating bindings..."
	cargo build
	cargo run --bin uniffi-bindgen generate --library ./target/debug/libproofmanager.dylib --language swift --out-dir ./bindings
	rustup target add aarch64-apple-ios-sim
	rustup target add aarch64-apple-ios
	cargo build --release --target=aarch64-apple-ios-sim
	cargo build --release --target=aarch64-apple-ios

	@echo "Setting up XCFramework..."
	rm -rf "ios/ProofManager.xcframework"	
	@echo "Creating XCFramework..."
	xcodebuild -create-xcframework \
		-library ./target/aarch64-apple-ios-sim/release/libproofmanager.a -headers ./bindings \
		-library ./target/aarch64-apple-ios/release/libproofmanager.a -headers ./bindings \
		-output "ios/ProofManager.xcframework"

# Copy target - handles moving files to the correct locations
ios-copy:
	@echo "Copying framework to modules..."
	rm -rf ../modules/proofmanager/ios/ProofManager.xcframework
	cp -r ios/ProofManager.xcframework ../modules/proofmanager/ios
	@echo "Cleaning up and copying Swift files..."
	rm -f ../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64/Headers/proofmanager.swift
	rm -f ../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64-simulator/Headers/proofmanager.swift
	rm -f ../modules/proofmanager/ios/proofmanager.swift
	cp ./bindings/proofmanager.swift ../modules/proofmanager/ios/

# Android Build and Copy
android: android-build android-copy

android-build:
	@export ANDROID_NDK_HOME=$(ANDROID_NDK_HOME) \
	&& export AR=$(ANDROID_NDK_TOOLCHAIN)/bin/llvm-ar \
	&& export CC=$(ANDROID_NDK_TOOLCHAIN)/bin/$(ANDROID_TARGET)$(ANDROID_API)-clang \
	&& cargo build --target $(ANDROID_TARGET) --release \
	&& cargo run --features=uniffi/cli --bin uniffi-bindgen generate \
		--library ./target/$(ANDROID_TARGET)/release/$(LIB_NAME).so \
		--language kotlin --out-dir $(BINDINGS_DIR)

android-copy:
	mkdir -p $(ANDROID_ABI_DIR) $(ANDROID_KOTLIN_DIR) $(ANDROID_MODULE_DIR)

	cp ./target/$(ANDROID_TARGET)/release/$(LIB_NAME).so $(ANDROID_ABI_DIR)/

	@if [ -f "$(BINDINGS_DIR)/uniffi/$(PROJECT_NAME)/$(PROJECT_NAME).kt" ]; then \
		cp $(BINDINGS_DIR)/uniffi/$(PROJECT_NAME)/$(PROJECT_NAME).kt $(ANDROID_KOTLIN_DIR)/; \
	else \
		echo "$(YELLOW)Warning: proofmanager.kt not found$(RESET)"; exit 1; \
	fi

	@if [ -f "$(MODULE_SOURCE)" ]; then \
		cp $(MODULE_SOURCE) $(ANDROID_MODULE_DIR)/; \
		echo "$(GREEN)✓ Module file copied$(RESET)"; \
	else \
		echo "$(YELLOW)Warning: ProofManagerModule.kt not found$(RESET)"; exit 1; \
	fi

	@echo "$(GREEN)✓ Android artifacts copied$(RESET)"

# Clean Targets
clean: ios-clean android-clean

android-clean:
	rm -rf ./target/$(ANDROID_TARGET) $(BINDINGS_DIR) $(ANDROID_ABI_DIR) $(ANDROID_KOTLIN_DIR) $(ANDROID_MODULE_DIR)
	@echo "$(GREEN)✓ Android clean completed$(RESET)"

ios-clean:
	@echo "Cleaning build artifacts..."
	rm -rf ./target
	rm -rf ./bindings
	rm -rf ./ios/ProofManager.xcframework
	rm -rf ../modules/proofmanager/ios/ProofManager.xcframework

# Build Summary
summary:
	@echo "$(BOLD)Build Summary$(RESET)"
	@echo "----------------------------------------"
	@if [ -e "$(ANDROID_ABI_DIR)/$(LIB_NAME).so" ]; then \
		echo "$(GREEN)✓ Android library built at: $(ANDROID_ABI_DIR)/$(LIB_NAME).so"; \
	fi
	@if [ -e "$(IOS_FRAMEWORK_PATH)" ]; then \
		echo "$(GREEN)✓ iOS framework built at: $(IOS_FRAMEWORK_PATH)"; \
	fi

# Help
help:
	@echo "$(BOLD)ProofManager Build System$(RESET)"
	@echo "Available commands:"
	@echo "  $(GREEN)make$(RESET)               Build everything (iOS and Android)"
	@echo "  $(GREEN)make ios$(RESET)           Build only iOS framework"
	@echo "  $(GREEN)make android$(RESET)       Build only Android library"
	@echo "  $(GREEN)make clean$(RESET)         Clean all build artifacts"
	@echo "  $(GREEN)make check$(RESET)         Verify build environment"
	@echo "  $(GREEN)make help$(RESET)          Show this help message"
