import os
import shutil
import re

def create_module_dirs(root_dir, modules):
    for module in modules:
        module_dir = os.path.join(root_dir, *module.split('/')[:-1])
        os.makedirs(module_dir, exist_ok=True)

def copy_content_to_files(src_file):
    with open(src_file, 'r') as f:
        lines = f.readlines()

    module_regex = r'//\s*File:\s*(\S+)'
    current_module = None
    module_files = {}
    content_buffer = []

    for line in lines:
        match = re.match(module_regex, line)
        if match:
            if current_module:
                module_files[current_module] = content_buffer
                content_buffer = []

            current_module = match.group(1)
            if current_module not in module_files:
                module_files[current_module] = []
        elif current_module:
            content_buffer.append(line)

    for module, content in module_files.items():
        # Remove duplicate 'src' directories from the module path
        module_path = os.path.normpath(os.path.join('src', module.split('/src2/')[1]))
        print(f"Copying content to: {module_path}")  # Informative message
        try:
            with open(module_path, 'w') as f:
                f.write(''.join(content))
        except FileNotFoundError:
            print(f"Error: Directory not found for module {module_path}. Ensure the directory structure exists.")
        except Exception as e:
            print(f"Error occurred while writing to {module_path}: {e}")

def main():
    root_dir = os.getcwd()
    src_file = os.path.join(root_dir, 'output2.rs')

    copy_content_to_files(src_file)

    print("Rust files organized successfully.")

if __name__ == "__main__":
    main()
