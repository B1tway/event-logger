# event-logger
----
**event-logger** is a tool for real-time tracing of user actions that can be used to create a dataset. 
**event-logger** collects information about all mouse movements, and mouse and keyboard button clicks, and also takes a screenshot of the user's work screen for each state change.

By logging all user actions, this data can be converted into a suitable format at the post-processing stage

## Build
```
cargo build --release
```
## Usase
### Run tool
```
event-logger.exe -d path_to_directory 
```
After launching the utility will be in standby mode, to enable logging of actions you need to press `Ctrl + Alt + P`.
After that, the tool will start saving all user actions to the appropriate directory
### Example of output data
```json
[
    {
        "time": {
            "secs_since_epoch": 1694212688,
            "nanos_since_epoch": 823792800
        },
        "name": null,
        "event_type": {
            "ButtonRelease": "Left"
        }
    }
]

```
### Usage of a postprocessing script
The post-processing script can be used to convert the data into a suitable format or to remove unnecessary records.
```
python post_process.py
```


