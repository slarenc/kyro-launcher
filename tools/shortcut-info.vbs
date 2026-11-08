Set shell = CreateObject("WScript.Shell")
Set link = shell.CreateShortcut(WScript.Arguments(0))
Function CodeUnits(value)
  Dim result, index, unit
  result = ""
  For index = 1 To Len(value)
    unit = AscW(Mid(value, index, 1))
    If unit < 0 Then unit = unit + 65536
    If index > 1 Then result = result & ","
    result = result & unit
  Next
  CodeUnits = result
End Function
WScript.Echo CodeUnits(link.TargetPath)
WScript.Echo CodeUnits(link.WorkingDirectory)
