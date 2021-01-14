import ctypes

def begin(rust):
	print(ctypes.c_long(rust.start()))