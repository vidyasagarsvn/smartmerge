from setuptools import setup, find_packages

setup(
    name="smartmerge",
    version="0.1.0",
    description="A cross-platform file and folder diff/merge tool with visual highlighting.",
    author="Your Name",
    packages=find_packages(),
    install_requires=[
        "pyside6"
    ],
    entry_points={
        "console_scripts": [
            "smartmerge = smartmerge.main:main"
        ]
    },
    include_package_data=True,
)
