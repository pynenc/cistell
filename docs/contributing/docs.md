# Documentation Setup for Cistell

This directory contains the Sphinx documentation for Cistell.

## Prerequisites

Install [uv](https://docs.astral.sh/uv/getting-started/installation/) and then install dependencies:

```bash
uv sync
```

## Building the Documentation

Once you have installed all the required dependencies, you can build the documentation:

1. **Navigate to the `docs` Directory**: Change your current directory to the `docs` subfolder:

   ```bash
   cd docs
   ```

2. **Build the Documentation**: Use the `make` utility to build the documentation:

   - On Unix-based systems (Linux, macOS), run:

     ```bash
     make html
     ```

   - On Windows, run:
     `bash .\make.bat html`
     This command will generate the HTML documentation in the `_build/html` directory.

   ```{tip}
   For examples check the [MyST syntax cheat sheet](https://jupyterbook.org/en/stable/reference/cheatsheet.html),
   [Roles and Directives](https://myst-parser.readthedocs.io/en/latest/syntax/roles-and-directives.html)
   and the complete directives references at [Myst docs](https://mystmd.org/guide/directives)
   ```

3. **View the Documentation**: Open the HTML files in the `_build/html` directory with a web browser to view the rendered documentation.

## Troubleshooting

- If you encounter any issues with missing dependencies or errors during the build process, ensure that all dependencies are correctly installed and that you are in the proper directory (where the Makefile is located).

- For detailed errors or troubleshooting, refer to the Sphinx documentation or check the console output for error messages.

For more information on how to `contribute` to the documentation, see the contributing section in the main project documentation.
