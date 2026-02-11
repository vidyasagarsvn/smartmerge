#!/usr/bin/env python3
"""Test copy operations directly."""

import sys
sys.path.insert(0, '/Users/vidyasagar/Documents/code/smartmerge_pyside')

from smartmerge.ui.file_compare_widget import FileCompareWidget
from PySide6.QtWidgets import QApplication

# Create app
app = QApplication(sys.argv)

# Create widget
widget = FileCompareWidget()

# Load files
widget.load_left_file('/Users/vidyasagar/Documents/code/smartmerge_pyside/examples/example3_left.txt')
widget.load_right_file('/Users/vidyasagar/Documents/code/smartmerge_pyside/examples/example3_right.txt')

# Print initial state
print(f"Initial: Regions={widget.get_change_count()}, Index={widget._current_region_index}")

# Try copies until it fails
for i in range(5):
    print(f"\nCopy attempt #{i+1}...")
    widget.copy_to_right()
    print(f"After: Regions={widget.get_change_count()}, Index={widget._current_region_index}")
    if widget.get_change_count() == 0:
        print("No more regions - stopping")
        break

