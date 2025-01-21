# Requirements

- Node
- Rust
    - iOS Targets
    - Android target
    - Cargo ndk
    - Cbindgen
- Xcode
- Android Studio
- CocoaPods

#  Setup

1. Create an expo app from scratch using 
	```npx create-expo-app -t expo-template-black-typescript expo-rust-demo```
2. Test to see if it work
	```
	cd expo-rust-demo	
	npm run iOS / npm run android
	```
3. Inside the `expo-rust-demo` folder create the scaffolding for the module
	```npx create-expo-module my-rust-module —local```

This will create the following folders:
	my-rust-module
		android (containes hello function)
		iOS
		src
		expo-module.config.json
		index.ts (it has the functions available to JS)


Let’s try to call the native module function. 
Before we can do that we need to run `npx expo prebuild`
Once prebuilt is done. 

Go to `App.tsx`: - Add the import from the module `import { hello } from ‘./modules/my-rust-module`- line 8 replace Hello world with `{ hello() }
Add `.prettierrc` inside just `{}`
Run `npm run iOS` again

# Rust library

In the root level folder, we create a new rust library with `cargo new —lib native_rust_lib`

The native_rust_lib folder will be created out of modules/ in the root folder itself.Add just one function that will be called from iOS and android.```rs
pub fn rust_add(left: i32, right: i32) -> i32 {
	left + right
	}
	```
	
Add `[#no_mangle]` over it so iOS can execute it too.

Add`extern “C”`Final `[#no_mangle]
pub extern “C” fn rust_add (left: i32, right: i32) -> i32 {
	left + right
}

Go to `native_rust_lib` Cargo.toml 

Add:

```
[lib]
crate-type = [“staticlib”,”cdylib”]

[dependencies]
jni = "0.21.1"
```

Static lib will create the .a file needed by iOS, cdylib will create the .so file needed by Android.

On [dependencies] add `jni = "0.21.1”` which is needed by Android

We have a function now and we need to build it for iOS

```
rustup target add aarch64-apple-ios aarch64-ios-sim 
```

Then we build it with for real devices and simulator with
 ```
 cargo build —release —target aarch64-apple-ios
 ```
 ```
 cargo build —release —target aarch64-apple-sim`
```

Then we need to generate a header so that iOS knows how to call the code.

We do it from inside the native_rust_lib folder using:
 ```cbindgen —lang c —crate native_rust_lib —output native_rust_lib.h```


To get iOS to work we then head to `my-rust-module/iOS/` we make a `rust` folder inside. 
Then we copy the `libnative_rust_lib.a` and `native_rust_lib.h` and paste both into the `my-rust-module/iOS/rust` folder.

Then we have to tell iOS `MyRustModule.podspec` to use our library by adding at the end of it 

```
s.vendored_libraries = ‘rust/libnative_rust_lib.a
```


Now on `MyRustModule.swift` we need to add our rust function with 

```
AsyncFunction(“rustAdd”) { (a: Int32, b: Int32) -> Int32 in 
	return rust_add(a,b)
}
```

Now go back to the root folder and let us install `cocoa pods` with

```
pod install —project-directory=iOS
```

# Exporting the function to react native
Next, we need to export our function to react native. 

We do this by editing the `index.ts` inside `my-rust-module`

```
export async function rustAdd(a: number, b: number ): Promise <number> {
	return await MyRustModule.rustAdd(a,b);
}
```

Back to `App.tsx` we can now integrate with the front-end by first adding it to the import statement

```
import { hello, rustAdd } from `./modules/my-rust-module’;
```
then 

```
export default function App() {
	const [value, setValue] = useState<null | number>(null);
	
	useEffect(() => 
	rustAdd(40,12)
	async function doFetch() {
	const result = await rustAdd(2,3)
	setValue(result);
}
	doFetch();
}, []);

	return (
	<View style={styles.container}>
	<Text style={styles.text}>
	{value === null ? “Loading…” : `The value is: ${value}`}
	</Text>
	</View>
);
}
```

Now we can test and see if it works `npm run iOS`# Android


# Scripts 
Adding scripts `cargo-iOS.ts` `cargo-android.ts` We need to add `npm i -D tsx` this enables us to run typescript files without compiling themGo to the root package.json and add the scripts```“cargo-ios”: “tsx scripts/cargo-ios.ts”“cargo-android”: “tax scripts/cargo-android.ts”


The `lib.rs` needs a review. 
Android needs a wrapping to make it understand 


Important naming convention for Android:Inside the `expo-module-config.json` the android modules format is important [“expo.modules.myrustmodule.MyRustModule”]


In lib.rs this will be called `Java_expo_module_myrustmodule_MyRustModule_rustAdd` 

Then we need to tell Kotlin how to call the lib and give back the result to Javascript.So Kotlin is inside /modules/my-rust-module/android/src/main/java/expo/modules/myrustmodule/MyRustModule.ktSteps for Kotlin1. Load the library	```	companion object {	
		init {
			System.loadLibrary(“native_rust_lib”)
		}	}	```
		
Next define the functions: 

``` 
external fun rustAdd (a: Int, b: Int): Int```

```
AsyncFunction(“rustAdd”) { a: Int, b: Int ->	rustAdd(a,b)
}

```