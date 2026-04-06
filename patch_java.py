#!/usr/bin/env python3

import os

def patch_file(path, search_pattern, insert_code, before=True):
    with open(path, 'r') as f:
        lines = f.readlines()
    
    new_lines = []
    found = False
    for line in lines:
        if not found and search_pattern in line:
            if before:
                new_lines.append(insert_code + '\n')
                new_lines.append(line)
            else:
                new_lines.append(line)
                new_lines.append(insert_code + '\n')
            found = True
        else:
            new_lines.append(line)
    
    with open(path, 'w') as f:
        f.writelines(new_lines)

path1 = "/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/InnerStateAutonom.java"
patch_file(path1, "final XDimension2D dim = text.mergeTB(attr, img);", 
           '                System.err.println("DEBUG_INNER_STATE: name=" + title + " title_h=" + text.getHeight() + " attr_h=" + attr.getHeight() + " img_h=" + img.getHeight());')
patch_file(path1, "final XDimension2D result = dim.delta", 
           '                System.err.println("DEBUG_INNER_STATE: merged_h=" + dim.getHeight() + " marginForFields=" + marginForFields);')
patch_file(path1, "return result;", 
           '                System.err.println("DEBUG_INNER_STATE: result_h=" + result.getHeight());')

path2 = "/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java"
with open(path2, 'r') as f:
    content = f.read()
content = content.replace("return minMax.getDimension().delta(15, 15);", 
                          '                XDimension2D res = minMax.getDimension().delta(15, 15);\n                System.err.println("DEBUG_SVEK_RESULT: lf_span_h=" + minMax.getDimension().getHeight() + " result_h=" + res.getHeight());\n                return res;')
with open(path2, 'w') as f:
    f.write(content)
