import unittest
from symbolize import Symbols, symbolize, self_samples

class SymbolTests(unittest.TestCase):
    def setUp(self):
        self.symbols=Symbols('1000 10 first\n1020 8 second\n')
    def test_bounds(self):
        self.assertIsNone(self.symbols.find(0xfff))
        self.assertEqual(self.symbols.find(0x1000),('first',0))
        self.assertEqual(self.symbols.find(0x100f),('first',15))
        self.assertIsNone(self.symbols.find(0x1010))
        self.assertIsNone(self.symbols.find(0x101f))
        self.assertIsNone(self.symbols.find(0x1028))
    def test_overlap_rejected(self):
        with self.assertRaises(ValueError):Symbols('1000 20 first\n1010 8 second\n')
    def test_zero_rejected(self):
        with self.assertRaises(ValueError):Symbols('1000 0 first\n')
    def test_resolve_preserves_unknown(self):
        text='???  (in <unknown binary>)  [0x1004]\n???  (in <unknown binary>)  [0x1015]'
        result,counts=symbolize(text,self.symbols,{'first':'Type.Method'})
        self.assertIn('Type.Method [first] (in JIT) + 4',result)
        self.assertIn('[0x1015]',result)
        self.assertEqual(counts,dict(resolvedOccurrences=1,unresolvedOccurrences=1))
    def test_self_samples(self):
        tree='Call graph:\n    10 root\n      + 6 child\n      + ! 2 leaf\n      + 3 other\nTotal number in stack'
        self.assertEqual(self_samples(tree),{'child':4,'other':3,'leaf':2,'root':1})
    def test_layout(self):
        self.assertEqual(self.symbols.layout(),dict(functions=2,codeBytes=24,addressSpan=40,codePages16KiB=1))

if __name__=='__main__':unittest.main()
