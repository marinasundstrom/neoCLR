"""Build bounded Raven-authored runtime implementations and its bootstrap snapshots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SLICES = {
    'GC': 'System.Runtime.GC',
    'DateTime': 'System.Time.DateTime',
    'ZonedDateTime': 'System.Time.ZonedDateTime',
    'LocalTimeMapping': 'System.Time.LocalTimeMapping',
    'TimeZoneError': 'System.Time.TimeZoneError',
    'TimeZone': 'System.Time.TimeZone',
    'TimeOffset': 'System.Time.TimeOffset',
    "Language": "System.Globalization.Language",
    "CultureProvider": "System.Globalization.CultureProvider",
    "FixedCultureProvider": "System.Globalization.FixedCultureProvider",
    "SystemCultureProvider": "System.Globalization.SystemCultureProvider",

    "Calendar": "System.Time.Calendar",
    "Culture": "System.Globalization.Culture",
    "DateTimeFormat": "System.Globalization.DateTimeFormat",
    "EqualityComparer": "System.Collections.EqualityComparer",
    "Comparer": "System.Collections.Comparer",
    "FunctionEqualityComparer": "System.Collections.FunctionEqualityComparer",
    "FunctionComparer": "System.Collections.FunctionComparer",
    "StringComparison": "System.StringComparison",
    "StringComparer": "System.StringComparer",

    "ReflectionError": "System.Runtime.Reflection.ReflectionError",
    'JsonError': 'System.Data.Json.JsonError',
    'HttpJsonError': 'System.Web.Http.Json.HttpJsonError',
    'JsonValue': 'System.Data.Json.JsonValue',
    "Cancellation": "System.Concurrency.CancellationTokenSource",
    "IPAddress": "System.Networking.IPAddress",
    "IPAddressError": "System.Networking.IPAddressError",
    "HttpError": "System.Web.Http.HttpError",
    "Uri": "System.Uri",
    "UriError": "System.UriError",
    "HttpClient": "System.Web.Http.HttpClient",
    "DnsError": "System.Networking.DnsError",
    "Dns": "System.Networking.Dns",
    "SocketError": "System.Networking.Sockets.SocketError",
    "Socket": "System.Networking.Sockets.Socket",
    'TextReadError': 'System.IO.TextReadError',
    'TextReader': 'System.IO.TextReader',
    'TextWriter': 'System.IO.TextWriter',
    'StreamWriter': 'System.IO.StreamWriter',
    'MemoryStream': 'System.IO.MemoryStream',
    'StreamReader': 'System.IO.StreamReader',
    'SeekableStream': 'System.IO.SeekableStream',

    'FileSystem': 'System.Storage.FileSystem',
    'StorageItems': 'System.Storage.StorageItem',
    'StorageProvider': 'System.Storage.StorageProvider',
    'InputStream': 'System.IO.InputStream',
    'OutputStream': 'System.IO.OutputStream',
    "InvalidPathError": "System.Storage.InvalidPathError",
    "EntryKind": "System.Storage.EntryKind",
    "HttpStatusCode": "System.Web.Http.HttpStatusCode",
    "StorageLookupError": "System.Storage.StorageLookupError",
    "StorageMetadata": "System.Storage.Metadata",
    'StreamError': 'System.IO.StreamError',
    'FileInputStream': 'System.IO.FileInputStream',
    'FileOutputStream': 'System.IO.FileOutputStream',

    'TaskOutcome': 'System.Concurrency.Tasks.TaskOutcome',
    'TaskState': 'System.Concurrency.Tasks.TaskState',
    'Tasks': 'System.Concurrency.Tasks.Task',
    'Workers': 'System.Concurrency.Thread',

    'Array': 'System.Array',
    'Object': 'System.Object',
    'HashCode': 'System.HashCode',
    'UnionAttribute': 'System.Runtime.CompilerServices.UnionAttribute',
    'BindingFlags': 'System.Introspection.BindingFlags',
    'Fault': 'System',
    'NativeMemory': 'System.Runtime.InteropServices.NativeMemory',
    'Descriptors': 'System.Introspection.MemberInfo',
    'Optional': 'System.Optional',
    'Result': 'System.Result',
    'Propagatable': 'System.Propagatable',
    'IntegerDivisionError': 'System.IntegerDivisionError',
    'SingleError': 'System.Linq.SingleError',
    'Number': 'System.Number',
    'NumberParseError': 'System.NumberParseError',
    'BooleanParseError': 'System.BooleanParseError',

    'Utf8SliceError': 'System.Text.Utf8SliceError',
    'ConsoleReadError': 'System.ConsoleReadError',
    'FileWriteError': 'System.Storage.FileWriteError',
    'FileReadError': 'System.Storage.FileReadError',
    'InvalidRangeError': 'System.InvalidRangeError',
    'InvalidDateError': 'System.Time.InvalidDateError',
    'InvalidTimeError': 'System.Time.InvalidTimeError',
    'OverflowError': 'System.OverflowError',
    'EnvironmentError': 'System.EnvironmentError',
    'Void': 'System.Void',

    'String': 'System.String',
    'Encoder': 'System.Text.Encoder',
    'EncoderProgress': 'System.Text.EncoderProgress',
    'EncoderState': 'System.Text.EncoderState',
    'Encoding': 'System.Text.Encoding',
    'Decoder': 'System.Text.Decoder',
    'EncodingError': 'System.Text.EncodingError',
    'Encodings': 'System.Text.Encodings',
    'StringBuilder': 'System.Text.StringBuilder',
    'Utf8': 'System.Text.Utf8',
    'UnicodeScalar': 'System.Text.UnicodeScalar',
    'InvalidUtf8Error': 'System.Text.InvalidUtf8Error',
    'Environment': 'System.Environment',
    'Console': 'System.Console',
    'IntPtr': 'System.IntPtr',
    'UIntPtr': 'System.UIntPtr',
    'Iterable': 'System.Collections.Iterable',
    'Iterator': 'System.Collections.Iterator',
    'Collection': 'System.Collections.Collection',
    'Sequence': 'System.Collections.Sequence',
    'MutableSequence': 'System.Collections.MutableSequence',
    'List': 'System.Collections.List',
    'Map': 'System.Collections.Map',
    'MutableMap': 'System.Collections.MutableMap',

    'SystemClock': 'System.Time.SystemClock',
    'LocalDateTime': 'System.Time.LocalDateTime',
    'Disposable': 'System.Disposable',
    'EquatableTo': 'System.EquatableTo',
    'ComparableTo': 'System.ComparableTo',
    'ConvertibleInto': 'System.ConvertibleInto',
    'Clonable': 'System.Clonable',
    'Closable': 'System.Closable',

    'Clock': 'System.Time.Clock',
    'AssemblyInfo': 'System.Introspection.AssemblyInfo',
    'ModuleInfo': 'System.Introspection.ModuleInfo',

    'ParameterInfo': 'System.Introspection.ParameterInfo',
    'Duration': 'System.Time.Duration',
    'Instant': 'System.Time.Instant',
    'RuntimeContext': 'System.Runtime.RuntimeContext',
    'RuntimeTypeHandle': "System.RuntimeTypeHandle",
    'Value': "System.Value",
    'Math': 'System.Math',
    'Linq': 'System.Linq.Operators',
    'OptionalOperators': 'System.OptionalOperators',
    'OptionalNestedOperators': 'System.OptionalNestedOperators',
    'ResultOperators': 'System.ResultOperators',
    'Int32': 'System.Int32',
    'Char': 'System.Char',
    'ArrayList': 'System.Collections.ArrayList',
    'HashMap': 'System.Collections.HashMap',
    'Time': 'System.Time.TimeOfDay',
    'Date': 'System.Time.Date',
    'Path': 'System.Storage.Path',
    'FileText': 'System.Storage.FileText',
    'Int64': 'System.Int64',
    'SByte': 'System.SByte',
    'Byte': 'System.Byte',
    'Int16': 'System.Int16',
    'UInt16': 'System.UInt16',
    'UInt32': 'System.UInt32',
    'UInt64': 'System.UInt64',
    'Single': 'System.Single',
    'Double': 'System.Double',
    'Boolean': 'System.Boolean',
}
SOURCES = {
    "Routes": "runtime/System.Web/src/System.Web/System/Web/Http/Routes.rvn",
    'GC': 'runtime/System.Runtime/src/System.Runtime/System/Runtime/GC.rvn',
    'DateTime': 'runtime/System.Runtime/src/System.Runtime/System/Time/DateTime.rvn',
    'ZonedDateTime': 'runtime/System.Runtime/src/System.Runtime/System/Time/ZonedDateTime.rvn',
    'LocalTimeMapping': 'runtime/System.Runtime/src/System.Runtime/System/Time/LocalTimeMapping.rvn',
    'TimeZoneError': 'runtime/System.Runtime/src/System.Runtime/System/Time/TimeZoneError.rvn',
    'TimeZone': 'runtime/System.Runtime/src/System.Runtime/System/Time/TimeZone.rvn',
    'TimeOffset': 'runtime/System.Runtime/src/System.Runtime/System/Time/TimeOffset.rvn',
    "Language": "runtime/System.Runtime/src/System.Runtime/System/Globalization/Language.rvn",
    "CultureProvider": "runtime/System.Runtime/src/System.Runtime/System/Globalization/CultureProvider.rvn",
    "FixedCultureProvider": "runtime/System.Runtime/src/System.Runtime/System/Globalization/FixedCultureProvider.rvn",
    "SystemCultureProvider": "runtime/System.Runtime/src/System.Runtime/System/Globalization/SystemCultureProvider.rvn",

    "InvariantDateTimeFormat": "runtime/System.Runtime/src/System.Runtime/System/Globalization/InvariantDateTimeFormat.rvn",
    "SwedishDateTimeFormat": "runtime/System.Runtime/src/System.Runtime/System/Globalization/SwedishDateTimeFormat.rvn",
    "HebrewDateTimeFormat": "runtime/System.Runtime/src/System.Runtime/System/Globalization/HebrewDateTimeFormat.rvn",
    "DateTimeFormatDigits": "runtime/System.Runtime/src/System.Runtime/System/Globalization/DateTimeFormatDigits.rvn",

    "GregorianCalendar": "runtime/System.Runtime/src/System.Runtime/System/Time/GregorianCalendar.rvn",
    "HebrewCalendar": "runtime/System.Runtime/src/System.Runtime/System/Time/HebrewCalendar.rvn",
    "Calendar": "runtime/System.Runtime/src/System.Runtime/System/Time/Calendar.rvn",
    "Culture": "runtime/System.Runtime/src/System.Runtime/System/Globalization/Culture.rvn",
    "DateTimeFormat": "runtime/System.Runtime/src/System.Runtime/System/Globalization/DateTimeFormat.rvn",
    "EqualityComparer": "runtime/System.Runtime/src/System.Runtime/System/Collections/EqualityComparer.rvn",
    "Comparer": "runtime/System.Runtime/src/System.Runtime/System/Collections/Comparer.rvn",
    "FunctionEqualityComparer": "runtime/System.Runtime/src/System.Runtime/System/Collections/FunctionEqualityComparer.rvn",
    "FunctionComparer": "runtime/System.Runtime/src/System.Runtime/System/Collections/FunctionComparer.rvn",
    "StringComparison": "runtime/System.Runtime/src/System.Runtime/System/StringComparison.rvn",
    "StringComparer": "runtime/System.Runtime/src/System.Runtime/System/StringComparer.rvn",

    "ReflectionError": "runtime/System.Runtime/src/System.Runtime/System/Runtime/Reflection/ReflectionError.rvn",
    "ReflectionExtensions": "runtime/System.Runtime/src/System.Runtime/System/Runtime/Reflection/ReflectionExtensions.rvn",
    'JsonValue': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonValue.rvn',
    'JsonError': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonError.rvn',
    'HttpJsonError': 'runtime/System.Web/src/System.Web/System/Web/Http/Json/HttpJsonError.rvn',
    'HttpClientJsonExtensions': 'runtime/System.Web/src/System.Web/System/Web/Http/Json/HttpClientJsonExtensions.rvn',
    'JsonClientOperations': 'runtime/System.Web/src/System.Web/System/Web/Http/Json/JsonClientOperations.rvn',
    'JsonContent': 'runtime/System.Web/src/System.Web/System/Web/Http/Json/JsonContent.rvn',
    'JsonDocument': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonDocument.rvn',
    'JsonSyntax': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonSyntax.rvn',
    'ObjectMapper': 'runtime/System.Data/src/System.Data/System/Data/Json/ObjectMapper.rvn',
    'JsonSerializer': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonSerializer.rvn',

    "Cancellation": "runtime/System.Runtime/src/System.Runtime/System/Concurrency/Cancellation.rvn",
    "IPAddress": "runtime/System.Networking/src/System.Networking/System/Networking/IPAddress.rvn",
    "IPAddressError": "runtime/System.Networking/src/System.Networking/System/Networking/IPAddressError.rvn",
    "Uri": "runtime/System.Runtime/src/System.Runtime/System/Uri.rvn",
    "UriError": "runtime/System.Runtime/src/System.Runtime/System/UriError.rvn",
    "HttpServer": "runtime/System.Web/src/System.Web/System/Web/Http/HttpServer.rvn",
    "HttpClient": "runtime/System.Web/src/System.Web/System/Web/Http/HttpClient.rvn",
    "HttpError": "runtime/System.Web/src/System.Web/System/Web/Http/HttpError.rvn",
    "DnsError": "runtime/System.Networking/src/System.Networking/System/Networking/DnsError.rvn",
    "Dns": "runtime/System.Networking/src/System.Networking/System/Networking/Dns.rvn",
    "SocketError": "runtime/System.Networking/src/System.Networking/System/Networking/Sockets/SocketError.rvn",
    "Socket": "runtime/System.Networking/src/System.Networking/System/Networking/Sockets/Socket.rvn",
    'TextReadError': 'runtime/System.Runtime/src/System.Runtime/System/IO/TextReadError.rvn',
    'TextReader': 'runtime/System.Runtime/src/System.Runtime/System/IO/TextReader.rvn',
    'TextWriter': 'runtime/System.Runtime/src/System.Runtime/System/IO/TextWriter.rvn',
    'StreamWriter': 'runtime/System.Runtime/src/System.Runtime/System/IO/StreamWriter.rvn',
    'MemoryStream': 'runtime/System.Runtime/src/System.Runtime/System/IO/MemoryStream.rvn',
    'StreamReader': 'runtime/System.Runtime/src/System.Runtime/System/IO/StreamReader.rvn',
    'SeekableStream': 'runtime/System.Runtime/src/System.Runtime/System/IO/SeekableStream.rvn',
    'FileSystem': 'runtime/System.Runtime/src/System.Runtime/System/Storage/FileSystem.rvn',
    'StorageItems': 'runtime/System.Runtime/src/System.Runtime/System/Storage/StorageItems.rvn',
    'StorageProvider': 'runtime/System.Runtime/src/System.Runtime/System/Storage/StorageProvider.rvn',
    'InputStream': 'runtime/System.Runtime/src/System.Runtime/System/IO/InputStream.rvn',
    'OutputStream': 'runtime/System.Runtime/src/System.Runtime/System/IO/OutputStream.rvn',
    "InvalidPathError": "runtime/System.Runtime/src/System.Runtime/System/Storage/InvalidPathError.rvn",
    "EntryKind": "runtime/System.Runtime/src/System.Runtime/System/Storage/EntryKind.rvn",
    "HttpStatusCode": "runtime/System.Web/src/System.Web/System/Web/Http/HttpStatusCode.rvn",
    "StorageLookupError": "runtime/System.Runtime/src/System.Runtime/System/Storage/StorageLookupError.rvn",
    "StorageMetadata": "runtime/System.Runtime/src/System.Runtime/System/Storage/Metadata.rvn",
    'StreamError': 'runtime/System.Runtime/src/System.Runtime/System/IO/StreamError.rvn',
    'FileInputStream': 'runtime/System.Runtime/src/System.Runtime/System/IO/FileInputStream.rvn',
    'FileOutputStream': 'runtime/System.Runtime/src/System.Runtime/System/IO/FileOutputStream.rvn',

    'TaskOutcome': 'runtime/System.Runtime/src/System.Runtime/System/Concurrency/Tasks/TaskOutcome.rvn',
    'TaskState': 'runtime/System.Runtime/src/System.Runtime/System/Concurrency/Tasks/TaskState.rvn',
    'Tasks': 'runtime/System.Runtime/src/System.Runtime/System/Concurrency/Tasks/Tasks.rvn',
    'Workers': 'runtime/System.Runtime/src/System.Runtime/System/Concurrency/Workers.rvn',

    'Array': 'runtime/System.Runtime/src/System.Runtime/System/Array.rvn',
    'Object': 'runtime/System.Runtime/src/System.Runtime/System/Object.rvn',
    'HashCode': 'runtime/System.Runtime/src/System.Runtime/System/HashCode.rvn',
    'UnionAttribute': 'runtime/System.Runtime/src/System.Runtime/System/Runtime/CompilerServices/UnionAttribute.rvn',
    'BindingFlags': 'runtime/System.Runtime/src/System.Runtime/System/Introspection/BindingFlags.rvn',
    'Fault': 'runtime/System.Runtime/src/System.Runtime/System/Functions.rvn',
    'NativeMemory': 'runtime/System.Runtime/src/System.Runtime/System/Runtime/InteropServices/NativeMemory/Functions.rvn',
    'Descriptors': 'runtime/System.Runtime/src/System.Runtime/System/Introspection/Descriptors.rvn',
    'Optional': 'runtime/System.Runtime/src/System.Runtime/System/Optional.rvn',
    'Result': 'runtime/System.Runtime/src/System.Runtime/System/Result.rvn',
    'Propagatable': 'runtime/System.Runtime/src/System.Runtime/System/Propagatable.rvn',
    'IntegerDivisionError': 'runtime/System.Runtime/src/System.Runtime/System/IntegerDivisionError.rvn',
    'SingleError': 'runtime/System.Runtime/src/System.Runtime/System/Linq/SingleError.rvn',
    'Number': 'runtime/System.Runtime/src/System.Runtime/System/Number.rvn',
    'NumberParseError': 'runtime/System.Runtime/src/System.Runtime/System/NumberParseError.rvn',
    'BooleanParseError': 'runtime/System.Runtime/src/System.Runtime/System/BooleanParseError.rvn',

    'Utf8SliceError': 'runtime/System.Runtime/src/System.Runtime/System/Text/Utf8SliceError.rvn',
    'ConsoleReadError': 'runtime/System.Runtime/src/System.Runtime/System/ConsoleReadError.rvn',
    'FileWriteError': 'runtime/System.Runtime/src/System.Runtime/System/Storage/FileWriteError.rvn',
    'FileReadError': 'runtime/System.Runtime/src/System.Runtime/System/Storage/FileReadError.rvn',
    'InvalidRangeError': 'runtime/System.Runtime/src/System.Runtime/System/InvalidRangeError.rvn',
    'InvalidDateError': 'runtime/System.Runtime/src/System.Runtime/System/Time/InvalidDateError.rvn',
    'InvalidTimeError': 'runtime/System.Runtime/src/System.Runtime/System/Time/InvalidTimeError.rvn',
    'OverflowError': 'runtime/System.Runtime/src/System.Runtime/System/OverflowError.rvn',
    'EnvironmentError': 'runtime/System.Runtime/src/System.Runtime/System/EnvironmentError.rvn',
    'Void': 'runtime/System.Runtime/src/System.Runtime/System/Void.rvn',

    'String': 'runtime/System.Runtime/src/System.Runtime/System/String.rvn',
    'Encoder': 'runtime/System.Runtime/src/System.Runtime/System/Text/Encoder.rvn',
    'EncoderProgress': 'runtime/System.Runtime/src/System.Runtime/System/Text/EncoderProgress.rvn',
    'EncoderState': 'runtime/System.Runtime/src/System.Runtime/System/Text/EncoderState.rvn',
    'Encoding': 'runtime/System.Runtime/src/System.Runtime/System/Text/Encoding.rvn',
    'Decoder': 'runtime/System.Runtime/src/System.Runtime/System/Text/Decoder.rvn',
    'EncodingError': 'runtime/System.Runtime/src/System.Runtime/System/Text/EncodingError.rvn',
    'Encodings': 'runtime/System.Runtime/src/System.Runtime/System/Text/Encodings.rvn',
    'StringBuilder': 'runtime/System.Runtime/src/System.Runtime/System/Text/StringBuilder.rvn',
    'Utf8': 'runtime/System.Runtime/src/System.Runtime/System/Text/Utf8.rvn',
    'UnicodeScalar': 'runtime/System.Runtime/src/System.Runtime/System/Text/UnicodeScalar.rvn',
    'InvalidUtf8Error': 'runtime/System.Runtime/src/System.Runtime/System/Text/InvalidUtf8Error.rvn',
    'Environment': 'runtime/System.Runtime/src/System.Runtime/System/Environment/Functions.rvn',
    'Console': 'runtime/System.Runtime/src/System.Runtime/System/Console/Console.rvn',
    'IntPtr': 'runtime/System.Runtime/src/System.Runtime/System/IntPtr.rvn',
    'UIntPtr': 'runtime/System.Runtime/src/System.Runtime/System/UIntPtr.rvn',
    'Iterable': 'runtime/System.Runtime/src/System.Runtime/System/Collections/Iterable.rvn',
    'Iterator': 'runtime/System.Runtime/src/System.Runtime/System/Collections/Iterator.rvn',
    'Collection': 'runtime/System.Runtime/src/System.Runtime/System/Collections/Collection.rvn',
    'Sequence': 'runtime/System.Runtime/src/System.Runtime/System/Collections/Sequence.rvn',
    'MutableSequence': 'runtime/System.Runtime/src/System.Runtime/System/Collections/MutableSequence.rvn',
    'List': 'runtime/System.Runtime/src/System.Runtime/System/Collections/List.rvn',
    'Map': 'runtime/System.Runtime/src/System.Runtime/System/Collections/Map.rvn',
    'MutableMap': 'runtime/System.Runtime/src/System.Runtime/System/Collections/MutableMap.rvn',

    'SystemClock': 'runtime/System.Runtime/src/System.Runtime/System/Time/SystemClock.rvn',
    'LocalDateTime': 'runtime/System.Runtime/src/System.Runtime/System/Time/LocalDateTime.rvn',
    'Disposable': 'runtime/System.Runtime/src/System.Runtime/System/Disposable.rvn',
    'EquatableTo': 'runtime/System.Runtime/src/System.Runtime/System/EquatableTo.rvn',
    'ComparableTo': 'runtime/System.Runtime/src/System.Runtime/System/ComparableTo.rvn',
    'ConvertibleInto': 'runtime/System.Runtime/src/System.Runtime/System/ConvertibleInto.rvn',
    'Clonable': 'runtime/System.Runtime/src/System.Runtime/System/Clonable.rvn',
    'Closable': 'runtime/System.Runtime/src/System.Runtime/System/Closable.rvn',

    'Clock': 'runtime/System.Runtime/src/System.Runtime/System/Time/Clock.rvn',
    'AssemblyInfo': 'runtime/System.Runtime/src/System.Runtime/System/Introspection/AssemblyInfo.rvn',
    'ModuleInfo': 'runtime/System.Runtime/src/System.Runtime/System/Introspection/ModuleInfo.rvn',

    'ParameterInfo': 'runtime/System.Runtime/src/System.Runtime/System/Introspection/ParameterInfo.rvn',
    'Duration': 'runtime/System.Runtime/src/System.Runtime/System/Time/Duration.rvn',
    'Instant': 'runtime/System.Runtime/src/System.Runtime/System/Time/Instant.rvn',
    'RuntimeContext': 'runtime/System.Runtime/src/System.Runtime/System/Runtime/RuntimeContext.rvn',
    'RuntimeTypeHandle': "runtime/System.Runtime/src/System.Runtime/System/RuntimeTypeHandle.rvn",
    'Value': "runtime/System.Runtime/src/System.Runtime/System/Value.rvn",
    'Math': 'runtime/System.Runtime/src/System.Runtime/System/Math/Functions.rvn',
    'Int32': 'runtime/System.Runtime/src/System.Runtime/System/Int32.rvn',
    'Char': 'runtime/System.Runtime/src/System.Runtime/System/Char.rvn',
    'Linq': 'runtime/System.Runtime/src/System.Runtime/System/Linq/Operators.rvn',
    'OptionalOperators': 'runtime/System.Runtime/src/System.Runtime/System/OptionalOperators.rvn',
    'OptionalNestedOperators': 'runtime/System.Runtime/src/System.Runtime/System/OptionalOperators.rvn',
    'ResultOperators': 'runtime/System.Runtime/src/System.Runtime/System/ResultOperators.rvn',
    'ArrayList': 'runtime/System.Runtime/src/System.Runtime/System/Collections/ArrayList.rvn',
    'HashMap': 'runtime/System.Runtime/src/System.Runtime/System/Collections/HashMap.rvn',
    'Time': 'runtime/System.Runtime/src/System.Runtime/System/Time/TimeOfDay.rvn',
    'Date': 'runtime/System.Runtime/src/System.Runtime/System/Time/Date.rvn',
    'Path': 'runtime/System.Runtime/src/System.Runtime/System/Storage/Path.rvn',
    'FileText': 'runtime/System.Runtime/src/System.Runtime/System/Storage/FileText.rvn',
    'Int64': 'runtime/System.Runtime/src/System.Runtime/System/Int64.rvn',
    'SByte': 'runtime/System.Runtime/src/System.Runtime/System/SByte.rvn',
    'Byte': 'runtime/System.Runtime/src/System.Runtime/System/Byte.rvn',
    'Int16': 'runtime/System.Runtime/src/System.Runtime/System/Int16.rvn',
    'UInt16': 'runtime/System.Runtime/src/System.Runtime/System/UInt16.rvn',
    'UInt32': 'runtime/System.Runtime/src/System.Runtime/System/UInt32.rvn',
    'UInt64': 'runtime/System.Runtime/src/System.Runtime/System/UInt64.rvn',
    'Single': 'runtime/System.Runtime/src/System.Runtime/System/Single.rvn',
    'Double': 'runtime/System.Runtime/src/System.Runtime/System/Double.rvn',
    'Boolean': 'runtime/System.Runtime/src/System.Runtime/System/Boolean.rvn',
}
PROJECT = ROOT / 'runtime/raven/System.Runtime.rvnproj'
GENERATED = ROOT / 'runtime/raven/generated'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def fragments(text, name="Math", owner="System.Math", bootstrap=False):
    if name == 'Path' and bootstrap:
        # The archived Neo profile has no Object or HashCode class contract.
        # Keep its existing lexical surface; Raven uses the complete implementation.
        text = text.replace('.extends System.Object\n', '').replace('.implements System.EquatableTo<System.Storage.Path>\n', '')
        text = text.replace('call instance System.Object::.ctor()', 'pop')
        text = re.sub(r'(?ms)^\.method instance override (?:Equals\(System.Object other\)|GetHashCode\(\))[^\n]*\n.*?^\.end\n', '', text)
        text = text.replace('.method instance override ToString()', '.method instance ToString()')
    lines = text.splitlines(keepends=True)
    methods, helpers, types = [], {}, []
    extension_owners = {}
    while lines:
        if not lines[0].strip():
            lines.pop(0)
            continue
        if lines[0].startswith(('.type ', '.interface ', '.delegate ')):
            depth = 0
            for index, line in enumerate(lines):
                token = line.strip().split(' ', 1)[0]
                if token in ('.type', '.interface', '.delegate', '.method', '.property'):
                    depth += 1
                elif token == '.end':
                    depth -= 1
                    if depth == 0:
                        break
            else:
                raise ValueError('Unclosed private implementation type')
            body = ''.join(lines[:index + 1])
            if lines[0].startswith(('.delegate ' + owner + '<', '.type class ' + owner + '<', '.type ' + owner + '<', '.interface ' + owner + '<')) or lines[0].strip() in ('.type ' + owner, '.interface ' + owner):
                methods.append(body)
            else:
                types.append(body)
            lines = lines[index + 1:]
            continue
        match = re.match(r'\.function (?:internal )?([^(]+)\(', lines[0])
        assert match, lines[0]
        end = lines.index('.end\n')
        body = ''.join(lines[:end + 1])
        if (name == 'Tasks' and match[1].startswith(('System.Concurrency.Tasks.TaskOperators.', 'System.Concurrency.Tasks.TaskResultOperators.'))) or ('<' not in match[1] and name == 'Descriptors' and match[1].startswith(('System.Runtime.Reflection.TypeReflectionExtensions.', 'System.Runtime.Reflection.PropertyReflectionExtensions.', 'System.Runtime.Reflection.ConstructorReflectionExtensions.', 'System.Runtime.Reflection.MethodReflectionExtensions.', 'System.Runtime.Reflection.FieldReflectionExtensions.'))):
            operator_owner = match[1].rsplit('.', 1)[0]
            extension_owners.setdefault(operator_owner, []).append(body.replace('.function ' + operator_owner + '.', '.method static ', 1))
        elif match[1].startswith(owner + '.'):
            # Retain the bootstrap owner used by direct IL and the archived Neo frontend.
            # Namespace functions use marked containers; static APIs retain their owner.
            if owner == "System":
                methods.append(body)
            elif body.startswith('.function internal '):
                methods.append(body.replace('.function internal ' + owner + '.', '.method internal static ', 1))
            else:
                methods.append(body.replace('.function ' + owner + '.', '.method static ', 1))
        else:
            assert match[1] not in helpers
            helpers[match[1]] = body
        lines = lines[end + 1:]
    for operator_owner, bodies in extension_owners.items():
        types.append('.type ' + operator_owner + '\n' + ''.join(bodies) + '.end\n')
    if name == 'Tasks':
        # Task and Task<T> share a source slice but have distinct nominal owners.
        submission = [body for body in methods if body.startswith('.method ')]
        methods = [body for body in methods if not body.startswith('.method ')]
        if submission:
            types.append('.type System.Concurrency.Tasks.Task\n' + ''.join(submission) + '.end\n')
    # Nongeneric classes can also own static factories. Merge their function roots
    # into the emitted class rather than leaving top-level method fragments.
    for body in types[:]:
        if body.startswith(('.type class ' + owner + '\n', '.type class abstract ' + owner + '\n')):
            types.remove(body)
            methods = [body[:-len('.end\n')] + ''.join(methods) + '.end\n']
    if name in ('SByte', 'Byte', 'Int16', 'UInt16', 'Int32', 'UInt32', 'Int64', 'UInt64', 'Single', 'Double', 'Boolean') and bootstrap:
        # The archived Neo profile has legacy carriers. New Result APIs belong
        # to the Raven library; do not add another manual carrier to that profile.
        methods = [re.sub(r'(?ms)^\.method static Parse\(.*?^\.end\n', '', body) if name != 'Int32' else body for body in methods]
        methods = [re.sub(r'(?m)^\.implements System\.Number(?:<[^>]+>)?$', '', body) for body in methods]
    if name == 'String' and bootstrap:
        # Object interpolation belongs to the Raven profile. The archived Neo
        # profile has no System.Object contract; prune its adapter transitively too.
        methods = [re.sub(r'(?ms)^\.method static Concat\(System\.Object .*?^\.end\n', '', body) for body in methods]
        # Explicit comparison modes are a Raven-profile API.
        methods = [re.sub(r'(?ms)^\.method static (?:Compare|CompareOrdinalIgnoreCase)\(.*?^\.end\n', '', body) for body in methods]
        # The archived Neo profile uses borrowed collection interfaces. Keep its
        # scalar-independent String operations; Raven uses the complete source.
        methods = [re.sub(r'(?ms)^\.method (?:private )?(?:internal static |instance (?:readonly byref )?)(?:GetIterator|GetScalars|CreateFromCharacters|CollectionCount)\(.*?^\.end\n', '', body)
                   for body in methods]
        methods = [re.sub(r'(?m)^\.implements System\.Collections\.(?:Iterable|Collection|Sequence)<Char>\n', '', body)
                   for body in methods]
        methods = [re.sub(r'(?ms)^\.property instance Metadata_[^\n]+\n\.get instance System\.String::CollectionCount\(\)\n\.end\n', '', body)
                   for body in methods]
    if name == 'Int32' and bootstrap:
        # Preserve the archived Neo API while the Raven API shares NumberParseError.
        methods = [body.replace('NumberParseError', 'Int32ParseError') for body in methods]
        helpers = {key.replace('NumberParseError', 'Int32ParseError'): body.replace('NumberParseError', 'Int32ParseError') for key, body in helpers.items()}
    # Retain only transitively called adapters; no application entry-point shim.
    used = set()
    pending = re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', ''.join(methods + types))
    if name == 'Descriptors':
        pending.append('System.Runtime.Reflection.TypeReflectionExtensions.CreateInstanceTyped<T0>')
    if name == 'JsonValue':
        # Public generic static wrappers use the existing generic-function ABI.
        # They are application roots even when no other library method calls them.
        pending.extend(helper for helper in helpers if helper.startswith('System.Web.Http.Json.HttpClientJsonExtensions.') or helper in (
            'System.Data.Json.JsonSerializer.DeserializeText<T0>',
            'System.Data.Json.JsonSerializer.DeserializeStream<T0>',
            'System.Web.Http.Json.JsonContent.ReadModel<T0>',
            'System.Web.Http.Json.JsonClientOperations.Convert<T0>'))
    while pending:
        helper = pending.pop()
        if helper in used or helper not in helpers:
            continue
        used.add(helper)
        pending.extend(re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', helpers[helper]))
    banner = f'; Generated from {SOURCES[name]}. Regenerate with build_runtime_library.py.\n'
    prefix = name + ('.bootstrap' if bootstrap else '')
    result = {prefix + '.methods.neoil': banner + ''.join(methods),
              prefix + '.helpers.neoil': banner + ''.join(body for name, body in helpers.items() if name in used) + ''.join(types)}
    if name in ('String', 'Path', 'SByte', 'Byte', 'Int16', 'UInt16', 'Int32', 'UInt32', 'Int64', 'UInt64', 'Single', 'Double', 'Boolean') and not bootstrap:
        result.update(fragments(text, name, owner, bootstrap=True))
    return result

def check_snapshot():
    for name in SLICES:
        data = json.loads((GENERATED / (name + '.json')).read_text())
        missing = set(SOURCES.values()) - set(data['inputs'])
        if missing:
            raise SystemExit('Missing Raven library inputs in ' + name + ': ' + ', '.join(sorted(missing)))
        for path, expected in data['inputs'].items():
            if digest(ROOT / path) != expected:
                raise SystemExit('Stale Raven library input: ' + path)
        for path, expected in data['outputs'].items():
            if digest(GENERATED / path) != expected:
                raise SystemExit('Modified generated library: ' + path)
    print('Raven library source and bootstrap snapshot hashes match')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--bridge', type=Path)
    parser.add_argument('--slice', action='append', choices=tuple(SLICES), help='Regenerate only selected implementation slices; omitted means all')
    parser.add_argument('--check', action='store_true', help='Regenerate and compare without modifying the snapshot')
    parser.add_argument('--check-snapshot', action='store_true', help='Check source/artifact hashes without an SDK')
    args = parser.parse_args()
    if args.check_snapshot:
        check_snapshot()
        return
    if not args.compiler or not args.bridge:
        parser.error('--compiler and --bridge are required for regeneration')
    with tempfile.TemporaryDirectory(prefix='neoclr-runtime-library-') as temporary:
        root = Path(temporary)
        (root / 'demo').mkdir()
        core = root / 'demo/NeoCLR.CoreProbe.dll'
        bridge = ['dotnet', str(args.bridge.resolve())]
        subprocess.run([*bridge, '--reference-library-core', str(core)], check=True)
        subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                        '-o', str(root / 'compiled')], env={**os.environ, 'NeoCLRBootstrapRoot': str(root), 'NeoCLRLibrarySlice': ''}, check=True)
        if args.check:
            check_snapshot()
        generated = {}
        for name, owner in SLICES.items():
            if args.slice and name not in args.slice:
                continue
            compiled = root / 'compiled'
            if name not in ('Math', 'Linq', 'OptionalOperators', 'OptionalNestedOperators', 'ResultOperators'):
                compiled = root / ('compiled-' + name)
                subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                                '-o', str(compiled)], env={**os.environ, 'NeoCLRBootstrapRoot': str(root),
                                'NeoCLRLibrarySlice': name}, check=True)
            imported = root / ('imported-' + name)
            subprocess.run([*bridge, '--library-implementation', str(compiled / 'System.Runtime.dll'),
                            str(core), owner, str(imported)], check=True)
            outputs = fragments((imported / 'Implementation.neoil').read_text(), name, owner)
            if args.check:
                for output, text in outputs.items():
                    if (GENERATED / output).read_text() != text:
                        raise SystemExit('Regenerated library differs: ' + output)
                continue
            inputs = [ROOT / path for path in SOURCES.values()]
            inputs += [ROOT / "runtime/System.Runtime/src/System.Runtime/System/Concurrency/Tasks/TaskOperators.rvn", ROOT / "runtime/System.Runtime/src/System.Runtime/System/Console/Streams.rvn"]
            inputs += [PROJECT, ROOT / 'build/NeoCLR.Raven.props']
            data = {'format': 'raven-library-bootstrap-v1', 'owner': owner,
                    'inputs': {str(p.relative_to(ROOT)): digest(p) for p in inputs},
                    'outputs': {output: hashlib.sha256(text.encode()).hexdigest() for output, text in outputs.items()},
                    'compilerSha256': digest(args.compiler), 'coreSha256': digest(core),
                    'exports': re.findall(r'(?m)^\.method (?:static |instance )(.+)', outputs[name + '.methods.neoil'])}
            generated.update(outputs)
            generated[name + '.json'] = json.dumps(data, indent=2) + '\n'
        # Do not publish a partial snapshot when a later implementation fails admission.
        GENERATED.mkdir(exist_ok=True)
        for output, text in generated.items():
            (GENERATED / output).write_text(text)
        print('Clean bootstrap regeneration matches checked-in implementation' if args.check else 'Generated Raven library bootstrap fragments')

if __name__ == '__main__':
    main()
