#!/bin/bash

###############################################################################
#
# source: https://github.com/mrbaseman/parse_yaml.git
#
###############################################################################
# Parses a YAML file and outputs variable assigments.  Can optionally accept a 
# variable name prefix and a variable name separator
#
# Usage:
#   parse_yaml file [prefix] [separator]
###############################################################################

function parse_yaml {
    unset i
    unset fs
    local prefix=$2
    local separator=${3:-_}

    local indexfix=-1
    # Detect awk flavor
    if awk --version 2>&1 | grep -q "GNU Awk" ; then
        # GNU Awk detected
        indexfix=-1
    elif awk -Wv 2>&1 | grep -q "mawk" ; then
        # mawk detected
        indexfix=0
    fi

    local s='[[:space:]]*' sm='[ \t]*' w='[a-zA-Z0-9_.]*' fs=${fs:-$(echo @|tr @ '\034')} i=${i:-  }

    ###############################################################################
    # cat:   read the yaml file into the stream
    # awk 1: process multi-line text
    # sed 1: remove comments and empty lines
    # sed 2: process lists
    # sed 3: process dictionaries
    # sed 4: rearrange anchors
    # sed 5: remove '---'/'...'/quotes, add file separator to create fields for awk 2
    # awk 2: convert the formatted data to variable assignments
    ###############################################################################

    cat $1 | \
    awk -F$fs "{multi=0;
        if(match(\$0,/$sm\|$sm$/)){multi=1; sub(/$sm\|$sm$/,\"\");}
        if(match(\$0,/$sm>$sm$/)){multi=2; sub(/$sm>$sm$/,\"\");}
        while(multi>0){
            str=\$0; gsub(/^$sm/,\"\", str);
            indent=index(\$0,str);
            indentstr=substr(\$0, 0, indent+$indexfix) \"$i\";
            obuf=\$0;
            getline;
            while(index(\$0,indentstr)){
                obuf=obuf substr(\$0, length(indentstr)+1);
                if (multi==1){obuf=obuf \"\\\\n\";}
                if (multi==2){
                    if(match(\$0,/^$sm$/))
                        obuf=obuf \"\\\\n\";
                        else obuf=obuf \" \";
                }
                getline;
            }
            sub(/$sm$/,\"\",obuf);
            print obuf;
            multi=0;
            if(match(\$0,/$sm\|$sm$/)){multi=1; sub(/$sm\|$sm$/,\"\");}
            if(match(\$0,/$sm>$sm$/)){multi=2; sub(/$sm>$sm$/,\"\");}
        }
    print}" | \
    sed -e "s|^\($s\)?|\1-|" \
        -ne "s|^\($s\)-$s\($w\)$s:$s\(.*\)|\1-\n\1 \2: \3|" \
        -ne "s|^$s#.*||;s|$s#[^\"']*$||;s|^\([^\"'#]*\)#.*|\1|;t 1" \
        -ne "t" \
        -ne ":1" \
        -ne "s|^$s\$||;t 2" \
        -ne "p" \
        -ne ":2" \
        -ne "d" | \
    sed -ne "s|,$s\]|]|g" \
        -e ":1" \
        -e "s|^\($s\)\($w\)$s:$s\(&$w\)$s\[$s\(.*\)$s,$s\(.*\)$s\]|\1\2: \3[\4]\n\1$i- \5|;t 1" \
        -e "s|^\($s\)\($w\)$s:$s\(&$w\)$s\[$s\(.*\)$s\]|\1\2: \3\n\1$i- \4|;" \
        -e ":2" \
        -e "s|^\($s\)\($w\)$s:$s\[$s\(.*\)$s,$s\(.*\)$s\]|\1\2: [\3]\n\1$i- \4|;t 2" \
        -e "s|^\($s\)\($w\)$s:$s\[$s\(.*\)$s\]|\1\2:\n\1$i- \3|;" \
        -e ":3" \
        -e "s|^\($s\)-$s\[$s\(.*\)$s,$s\(.*\)$s\]|\1- [\2]\n\1$i- \3|;t 3" \
        -e "s|^\($s\)-$s\[$s\(.*\)$s\]|\1-\n\1$i- \2|;p" | \
    sed -ne "s|,$s}|}|g" \
        -e ":1" \
        -e "s|^\($s\)-$s{$s\(.*\)$s,$s\($w\)$s:$s\(.*\)$s}|\1- {\2}\n\1$i\3: \4|;t 1" \
        -e "s|^\($s\)-$s{$s\(.*\)$s}|\1-\n\1$i\2|;" \
        -e ":2" \
        -e "s|^\($s\)\($w\)$s:$s\(&$w\)$s{$s\(.*\)$s,$s\($w\)$s:$s\(.*\)$s}|\1\2: \3 {\4}\n\1$i\5: \6|;t 2" \
        -e "s|^\($s\)\($w\)$s:$s\(&$w\)$s{$s\(.*\)$s}|\1\2: \3\n\1$i\4|;" \
        -e ":3" \
        -e "s|^\($s\)\($w\)$s:$s{$s\(.*\)$s,$s\($w\)$s:$s\(.*\)$s}|\1\2: {\3}\n\1$i\4: \5|;t 3" \
        -e "s|^\($s\)\($w\)$s:$s{$s\(.*\)$s}|\1\2:\n\1$i\3|;p" | \
    sed -e "s|^\($s\)\($w\)$s:$s\(&$w\)\(.*\)|\1\2:\4\n\3|" \
        -e "s|^\($s\)-$s\(&$w\)\(.*\)|\1- \3\n\2|" | \
    sed -ne "s|^\($s\):|\1|" \
        -e "s|^\($s\)\(---\)\($s\)||" \
        -e "s|^\($s\)\(\.\.\.\)\($s\)||" \
        -e "s|^\($s\)-$s[\"']\(.*\)[\"']$s\$|\1$fs$fs\2|p;t" \
        -e "s|^\($s\)\($w\)$s:$s[\"']\(.*\)[\"']$s\$|\1$fs\2$fs\3|p;t" \
        -e "s|^\($s\)-$s\(.*\)$s\$|\1$fs$fs\2|" \
        -e "s|^\($s\)\($w\)$s:$s[\"']\?\(.*\)$s\$|\1$fs\2$fs\3|" \
        -e "s|^\($s\)[\"']\?\([^&][^$fs]\+\)[\"']$s\$|\1$fs$fs$fs\2|" \
        -e "s|^\($s\)[\"']\?\([^&][^$fs]\+\)$s\$|\1$fs$fs$fs\2|" \
        -e "s|^\($s\)\($w\)$s:$s[\"']\(.*\)$s\$|\1$fs\2$fs\3|" \
        -e "s|^\($s\)[\"']\([^&][^$fs]*\)[\"']$s\$|\1$fs$fs$fs\2|" \
        -e "s|^\($s\)[\"']\([^&][^$fs]*\)$s\$|\1$fs$fs$fs\2|" \
        -e "s|^\($s\)\($w\)$s:$s\(.*\)$s\$|\1$fs\2$fs\3|" \
        -e "s|^\($s\)\([^&][^$fs]*\)[\"']$s\$|\1$fs$fs$fs\2|" \
        -e "s|^\($s\)\([^&][^$fs]*\)$s\$|\1$fs$fs$fs\2|" \
        -e "s|$s\$||p" | \
    awk -F$fs "{
        gsub(/\t/,\"        \",\$1);
        if(NF>3){if(value!=\"\"){value = value \" \";}value = value  \$4;}
        else {
            if(match(\$1,/^&/)){anchor[substr(\$1,2)]=full_vn;getline};
            indent = length(\$1)/length(\"$i\");
            vname[indent] = \$2;
            value= \$3;
            for (i in vname) {if (i > indent) {delete vname[i]; idx[i]=0}}
            if(length(\$2)== 0){  vname[indent]= ++idx[indent] };
            vn=\"\"; for (i=0; i<indent; i++) { vn=(vn)(vname[i])(\"$separator\")}
            vn=\"$prefix\" vn;
            full_vn=vn vname[indent];
            if(vn==\"$prefix\")vn=\"$prefix$separator\";
            if(vn==\"_\")vn=\"__\";
        }
        gsub(/\./,\"$separator\",full_vn);
	gsub(/\\\\\"/,\"\\\"\",value);
	gsub(/'/,\"'\\\"'\\\"'\",value);
        assignment[full_vn]=value;
        if(!match(assignment[vn], full_vn))assignment[vn]=assignment[vn] \" \" full_vn;
        if(match(value,/^\*/)){
            ref=anchor[substr(value,2)];
            if(length(ref)==0){
                printf(\"%s='%s'\n\", full_vn, value);
            } else {
                for(val in assignment){
                    if((length(ref)>0)&&index(val, ref)==1){
                        tmpval=assignment[val];
                        sub(ref,full_vn,val);
                        if(match(val,\"$separator\$\")){
                            gsub(ref,full_vn,tmpval);
                        } else if (length(tmpval) > 0) {
                            printf(\"%s='%s'\n\", val, tmpval);
                        }
                        assignment[val]=tmpval;
                    }
                }
            }
        } else if (length(value) > 0) {
            printf(\"%s='%s'\n\", full_vn, value);
        }
    }END{
        for(val in assignment){
            if(match(val,\"$separator\$\"))
                printf(\"%s='%s'\n\", val, assignment[val]);
        }
    }"
}
