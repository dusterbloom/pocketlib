# Start

Create a new `expo` app with 
```
npx create-expo-app@latest
```

Test if it works by running 
```
npm run android
```

# The expo module

Create a new expo module by  running
```
npx create-expo-module proofmanager --local
```

This below should be the output:
```
Need to install the following packages:
create-expo-module@0.8.9
Ok to proceed? (y) y

The local module will be created in the modules directory in the root of your project. Learn more: https://expo.fyi/expo-module-local-autolinking.md

? What is the name of the local module? › proofmanager
✔ What is the name of the local module? … proofmanager
✔ What is the native module name? … Proofmanager
✔ What is the Android package name? … expo.modules.proofmanager

✔ Downloaded module template from npm
✔ Created the module from template files

✅ Successfully created Expo module in modules/proofmanager
```






# Testing connection between module and app

Let’s try to call the native module function. Before we can do that we need to run `npx expo prebuild`
Once prebuilt is done. Go to `app/(tabs)/index.tsx`: 

- Add the imports
```
import { NativeModule, EventEmitter, EventSubscription } from 'expo-modules-core';

import ProofmanagerModule from './src/ProofmanagerModule';
import ProofmanagerModuleView from './src/ProofmanagerView'

export const PI = ProofmanagerModule.PI;


export function hello(): string {
    return ProofmanagerModule.hello(); 
}
```

If you want to test you can by changing `modules/proofmanager/android/src/main/java/expo/modules/proofmanager/ProofmanagerModule.kt`
```
    // Defines a JavaScript synchronous function that runs the native code on the JavaScript thread.
    Function("hello") {
      "Hello mondo!" // Changed 
    }
```

Add `.prettierrc` inside just `{}`
Run `npm run android` again


# Current app

In the current app we already have

1. `native_rust_lib`
2. `modules/proofmanager`
3. `app/(tabs)/index.tsx`

## Native Rust Library folder 

Inside `native_rust_lib` we have a `makefile` which can be used to `make android` as the android one is the most uptodate
This command will generate the target library for android and the kotlin bindings both of which will be automatically copied inside the 
`modules/proofmanager/android/src/main/uniffi/proofmanager/proofmanager.kt` and `modules/proofmanager/android/src/main/jniLibs/arm64-v8a/libproofmanager.so`

It is wise to delete the `bindings` folder from `native_rust_lib` after any change to the lib.rs

## Modules folder

