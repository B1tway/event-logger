import os
import json
import csv


x_last = 0
y_last = 0



def get_button_str(button):
    if button == 'Left':
        return 'Left'
    elif button == 'Right':
        return 'Right'
    elif button == 'Middle':
        return 'Scroll'

is_pressed = False
def parse_event(writer, event):
    global is_pressed, x_last, y_last
    timestamp = event['time']['secs_since_epoch']
    e = event['event_type']
    if 'MouseMove' in e:
        x_last = e['MouseMove']['x']
        y_last = e['MouseMove']['y']
        state = 'Drag' if is_pressed else 'Move'
        record = [timestamp, timestamp, 'NoButton', state, int(x_last), int(y_last)]
        writer.writerow(record)
        print(record)
    elif 'ButtonPress' in e:
        is_pressed = True
        b_str = get_button_str(e['ButtonPress'])
        record = [timestamp, timestamp, b_str, 'Pressed', int(x_last), int(y_last)]
        print(record)
        writer.writerow(record)
    elif 'ButtonRelease' in e:
        is_pressed = False
        b_str = get_button_str(e['ButtonRelease'])
        record = [timestamp, timestamp, b_str, 'Released', int(x_last), int(y_last)]
        print(record)
        writer.writerow(record)


if __name__ == "__main__":
    directory = 'data'
    file_path = 'data.csv'
    headers = ["record timestamp", "client timestamp",
               "button", "state", "x", "y"]
    
    file =  open(file_path, 'w', newline='')

    writer = csv.writer(file)
    writer.writerow(headers)

    for filename in os.listdir(directory):
        if filename.endswith('.json'):
            file_path = os.path.join(directory, filename)
            with open(file_path, 'r') as file:
                json_data = json.load(file)
                if isinstance(json_data, list):
                    for data_record in json_data:
                        parse_event(writer, data_record)
                else:
                    parse_event(writer, json_data)
