import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('windows_comparison', Path(__file__).with_name('windows-root-observation.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class CompilerOptions(unittest.TestCase):
    def test_relocates_outputs_without_changing_input_objects(self):
        command = ['cl', '/FoD:\\old/', '/Fe:D:\\old\\app.exe', 'guest.obj', 'root-probe.c']
        rewritten = module.replace_option(module.replace_option(command, '/Fo', 'D:\\new/'), '/Fe:', 'D:\\new\\app.exe')
        self.assertEqual(rewritten, ['cl', '/FoD:\\new/', '/Fe:D:\\new\\app.exe', 'guest.obj', 'root-probe.c'])
        self.assertEqual(command[1], '/FoD:\\old/')

    def test_requires_unambiguous_recipe(self):
        for command in [['cl'], ['cl', '/Foone.obj', '/Fotwo.obj']]:
            with self.assertRaises(ValueError):
                module.replace_option(command, '/Fo', 'new.obj')

    def test_link_output_is_case_insensitive(self):
        self.assertEqual(module.replace_option(['link', '/out:old.dll', 'a.obj'], '/OUT:', 'new.dll'),
                         ['link', '/OUT:new.dll', 'a.obj'])
