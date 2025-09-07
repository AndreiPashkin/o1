#!/usr/bin/env python3
"""Generates data for testing the compile-time maps."""
import json
import random
import string


def generate_random_string(min_length=3, max_length=255):
    """Generate a random string of variable length."""
    length = random.randint(min_length, max_length)
    return ''.join(random.choice(string.ascii_letters + string.digits) for _ in
                   range(length))


def generate_random_value():
    """Generate a random value for the data entries."""
    return random.randint(0, 9999999999)


def main():
    types = [
        ("u8", 0, 255, 255),  # Full range for u8
        ("i8", -128, 127, 255),  # Full range for i8
        ("u16", 0, 65535, 999),
        ("i16", -32768, 32767, 999),
        ("u32", 0, 4294967295, 999),
        ("i32", -2147483648, 2147483647, 999),
        ("u64", 0, 4611686018427387902, 999),
        ("i64", -2305843009213693951, 2305843009213693950, 999),
        ("u128", 0, 4611686018427387902, 999),
        ("i128", -2305843009213693951, 2305843009213693950, 999),
        ("usize", 0, 4611686018427387902, 999),
        ("isize", -2305843009213693951, 2305843009213693950, 999),
    ]

    result = {}
    for type_name, min_val, max_val, count in types:
        keys = set()
        while len(keys) < count:
            keys.add(random.randint(min_val, max_val))

        data = {}
        for key in keys:
            data[key] = generate_random_value()

        result[f"{type_name.upper()}_DATA"] = {
            "type": type_name,
            "data": data
        }

    string_data = {}
    str_count = 99

    unique_strings = set()
    while len(unique_strings) < str_count:
        unique_strings.add(generate_random_string(3, 15))

    for s in unique_strings:
        string_data[s] = generate_random_value()

    result["STR_DATA"] = {
        "type": "&str",
        "data": string_data
    }

    result["BOOL_DATA"] = {
        "type": "bool",
        "data": {
            "false": generate_random_value(),
            "true": generate_random_value()
        }
    }

    print(json.dumps(result, indent=4))


if __name__ == "__main__":
    main()
