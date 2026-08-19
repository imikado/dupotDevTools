#!/usr/bin/env python3
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from infrastructure.ui.app_window import AppWindow


def main():
    app = AppWindow()
    app.run(sys.argv)


if __name__ == "__main__":
    main()
