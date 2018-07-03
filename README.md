# Metroid(-like) OpenGL Demo
An OpenGL tech demo with a skybox, some geometry, bitmapped font rendering, and a moveable camera.

### Installation
Fork the demo and enter
```
cargo run
```
to run it.

### Dependencies
The demo may require an older version of Rust to run. If the program crashes from a uniform shader variable not being found, try running it with Rust 1.25.0. You can acquire it by running rustup:
```
rustup toolchain install 1.25.0
```

### Controls
The demo has the following control scheme.
* A -- Move camera left
* D -- Move camera right
* Q -- Move camera up
* E -- Move camera down
* W -- Move camera forward
* S -- Move camera backwards
* Left Arrow -- Rotate camera left
* Right Arrow -- Rotate camera right
* Up Key -- Rotate camera up
* Down Key -- Rotate camera down
* Z -- Roll camera left
* C -- Roll camera right
* Escape -- Close window and shut down program
