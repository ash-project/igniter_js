from setuptools import setup, find_packages

setup(
    name="css_tools",
    version="0.1.0",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    install_requires=[
        "tinycss2>=1.4.0",
    ],
)
