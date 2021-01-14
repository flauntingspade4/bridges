import ctypes

def begin(rust):
	rust.start()

if __name__ == "__main__":
	rust = ctypes.CDLL("rust/target/release/rust.dll")
	begin(rust)
