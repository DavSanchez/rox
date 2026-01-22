#!/usr/bin/env nu

def main [
    --interpreter: string
    --test-suite: string
    ...chapters: string
] {
    let chapter_map = {
        "14": "chap14_chunks",
        "15": "chap15_virtual",
        "16": "chap16_scanning",
        "17": "chap17_compiling",
        "18": "chap18_types",
        "19": "chap19_strings",
        "20": "chap20_hash",
        "21": "chap21_global",
        "22": "chap22_local",
        "23": "chap23_jumping",
        "24": "chap24_calls",
        "25": "chap25_closures",
        "26": "chap26_garbage",
        "27": "chap27_classes",
        "28": "chap28_methods",
        "29": "chap29_superclasses",
        "30": "chap30_optimization"
    }

    let ordered_chapters = [
        "chap14_chunks",
        "chap15_virtual",
        "chap16_scanning",
        "chap17_compiling",
        "chap18_types",
        "chap19_strings",
        "chap20_hash",
        "chap21_global",
        "chap22_local",
        "chap23_jumping",
        "chap24_calls",
        "chap25_closures",
        "chap26_garbage",
        "chap27_classes",
        "chap28_methods",
        "chap29_superclasses",
        "chap30_optimization"
    ]

    let targets = if ($chapters | is-empty) {
        $ordered_chapters
    } else {
        $chapters | each {|it|
            if ($it in $chapter_map) {
                $chapter_map | get $it
            } else {
                error make {msg: $"Error: Invalid chapter number provided: ($it)"}
            }
        }
    }

    # Create temp dir
    let tmp_dir = (mktemp -d | str trim)
    
    # We use a try/catch pattern for cleanup
    try {
        print $"Copying ($test_suite) to temporary directory ($tmp_dir)..."
        # Use external cp to handle permissions better (coreutils)
        cp -r $"($test_suite)/." $tmp_dir
        
        # Ensure we can write to the directory (files from nix store are read-only)
        chmod -R u+w $tmp_dir

        cd $"($tmp_dir)/tool"
        dart pub get
        cd $tmp_dir

        for target in $targets {
            print "--------------------------------------------------------------------------------"
            print $"Running ($target)..."
            print "--------------------------------------------------------------------------------"
            
            dart tool/bin/test.dart $target --interpreter $interpreter --arguments $"--($target)"
        }
    } catch {|err| 
        # Cleanup before exiting with error
        rm -rf $tmp_dir
        error make {msg: ($err.msg)}
    }
    
    # Cleanup on success
    rm -rf $tmp_dir
}
