import os

def get_rs_files(directory):
    rs_files = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith(".rs"):
                rs_files.append(os.path.join(root, file))
    return rs_files

def comment_file_content(filename):
    with open(filename, 'r') as f:
        content = f.read()
    return f"// File: {filename}\n{content}"

def main():
    directory = "./src"
    rs_files = get_rs_files(directory)
    comments = []
    for file in rs_files:
        comments.append(comment_file_content(file))

    with open("output.rs", "w") as output_file:
        for comment in comments:
            output_file.write(comment + "\n")

if __name__ == "__main__":
    main()
