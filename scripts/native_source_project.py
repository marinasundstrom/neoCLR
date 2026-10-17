"""Write a native sample project using a bundle's declared compiler contracts."""
from pathlib import Path
import xml.etree.ElementTree as ET


def create_project(directory, name, libraries, sources):
    directory = Path(directory).resolve()
    directory.mkdir(parents=True, exist_ok=True)
    project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
    ET.SubElement(project, 'Import', Project=str(Path(libraries).resolve() / 'NeoCLR.ClassLibrary.props'))
    properties = ET.SubElement(project, 'PropertyGroup')
    for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName=name,
                           EnableDefaultCompileItems='false').items():
        ET.SubElement(properties, key).text = value
    items = ET.SubElement(project, 'ItemGroup')
    for source in sources:
        ET.SubElement(items, 'Compile', Include=str(Path(source).resolve()))
    path = directory / (name + '.rvnproj')
    ET.indent(project)
    ET.ElementTree(project).write(path, encoding='unicode')
    return path, directory / 'bin/neoclr' / (name + '.dll')
