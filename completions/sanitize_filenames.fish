complete -c sanitize_filenames -s h -l help -d 'Show help message and exit'
complete -c sanitize_filenames -s r -l recursive -d 'Recursively sanitize directories and their contents'
complete -c sanitize_filenames -s n -l dry-run -d 'Show actions without renaming files'
complete -c sanitize_filenames -s c -l replacement -d 'Replacement character to use' -r -a '_ - . +'

