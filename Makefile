# Cairn build orchestration.
#
# Targets build the Rust engine and the Go wrapper, run both test suites, and
# run the end-to-end demo.

# Detect Windows for the correct binary extension.
ifeq ($(OS),Windows_NT)
	EXE := .exe
else
	EXE :=
endif
