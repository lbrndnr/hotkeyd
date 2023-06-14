# hotkeyd

`hotkeyd` allows you to easily register global keyboard shortcuts. It's designed to be lightweight and easily extensible. It currently supports macOS, but should be up and running on Linux or Windows relatively quickly.

## Usage

First, define a profile in your home directory. This example profile opens Visual Studio Code when pressing `⌥V`.
```
[
    {
        "key": "v",
        "modifiers": ["alt"],
        "command": "open /Applications/Visual\\ Studio\\ Code.app"
    }
]
```

Next, run `hotkeyd`. It will ask you for the Accessibility permission, which is necessary on macOS to observe the keyboard.

## License
`hotkeyd` is licensed under the [MIT License](http://opensource.org/licenses/mit-license.php).