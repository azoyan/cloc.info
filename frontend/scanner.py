import re
import os

def get_imports(file_name):
    with open(file_name, 'r') as file:
        content = file.read()
        pattern = r"import \{([^}]*)\} from './tailwind-classes.js'"
        match = re.search(pattern, content, re.DOTALL)
        if match:
            imports = match.group(1).replace('\n', '').replace(' ', '').split(',')
            return imports
        else:
            return []

def get_html_classes(file_name):
    with open(file_name, 'r') as file:
        content = file.read()
        pattern = r"class=['\"]([^'\"]+)['\"]"
        matches = re.findall(pattern, content)
        classes = []
        for match in matches:
            for class_name in match.split():
                class_name = class_name.replace(':', '_').replace('/','_').replace('[','').replace(']','')
                class_name = class_name.replace('-', '_').replace('.', '_').upper()
                if not (class_name.startswith('BI') or class_name.startswith('BI_')):
                    classes.append(class_name)
                else: print("ignored", class_name)
        return classes

def main():
    file_names = ['./src/js/info.js', './src/js/index.js', './src/js/api.js']
    html_file_names = ['./src/components/index.html', "./src/components/nav.html", "./src/components/info.html", "./src/components/footer.pug", "./src/layouts/main.pug", "./src/pages/index.pug", "./src/pages/info.pug"]
    all_imports = set()

    for file_name in file_names:
        imports = get_imports(file_name)
        all_imports.update(imports)

    for file_name in html_file_names:
        classes = get_html_classes(file_name)
        all_imports.update(classes)

    unique_imports = ', '.join(all_imports)
    print(f"import {{ {unique_imports} }} from './src/js/tailwind-classes.js'")

if __name__ == "__main__":
    main()
