import python
import ctypes

if __name__ == "__main__":
	rust = ctypes.CDLL("rust/target/release/rust.dll")
	python.begin(rust)
