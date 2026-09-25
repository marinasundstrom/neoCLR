using System;
using System.IO;
using System.Text.Json;

static void Check(bool condition) { if (!condition) throw new Exception("Reference check failed"); }
using var stream = new MemoryStream();
byte[] bytes = [65, 66, 67];
stream.Write(bytes);
stream.Seek(1, SeekOrigin.Begin);
stream.Write(bytes, 0, 1);
stream.Seek(5, SeekOrigin.Begin);
Check(stream.Length == 3);
stream.Write(bytes, 2, 1);
stream.Position = 0;
byte[] buffer = new byte[8];
Check(stream.Read(buffer, 1, 7) == 6);
Check(buffer[1] == 65 && buffer[2] == 65 && buffer[3] == 67);
Check(buffer[4] == 0 && buffer[5] == 0 && buffer[6] == 67);
Check(stream.Read(buffer, 0, 1) == 0);
using var json = new MemoryStream();
using var document = JsonDocument.Parse("{\"station\":\"Café 😀\",\"readings\":[21,22.5],\"ready\":true,\"note\":null}");
JsonSerializer.Serialize(json, document.RootElement);
Check(json.CanWrite);
json.Position = 0;
var restored = JsonSerializer.Deserialize<JsonElement>(json);
Check(json.CanRead && restored.GetProperty("station").GetString() == "Café 😀");
Console.WriteLine(".NET memory/JSON baseline passed");
