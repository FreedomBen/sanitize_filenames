#!/usr/bin/env ruby

#
# sanitize_filename.rb - A script to sanitize filenames to make them easier
#                        and safer to work with.  Pass in a list of filenames
#                        or directories and the script will rename them to
#                        remove whitespace and special characters.
#
# TODO
#
# 1.  Add a summary report of renamed files at the end of execution
# 2.  Consider exposing an allowlist of characters beyond simple replacement
#

require 'fileutils'
require 'optparse'
require 'tmpdir'
require 'stringio'

def capture_stdout
  original_stdout = $stdout
  $stdout = StringIO.new
  yield
  $stdout.string
ensure
  $stdout = original_stdout
end

def capture_stderr
  original_stderr = $stderr
  $stderr = StringIO.new
  yield
  $stderr.string
ensure
  $stderr = original_stderr
end

def has_dot?(filename) = filename.split('.').count > 1
def is_hidden?(filename) = filename.start_with?('.')
def is_directory?(filename) = File.directory?(filename)

def has_extension?(filename)
  has_dot?(filename) && !is_directory?(filename) && !is_hidden?(filename)
end

def extract_extension(filename)
  # if !has_dot?(filename) || is_directory?(filename)
  return filename.split('.').last if has_extension?(filename)
  ''
end

def sanitized_filename(input_file, replacement: '_')
  # If the file is directory, we will replace the trailing . since
  # dirs don't have file extensions
  extension = extract_extension(input_file)
  escaped_extension = Regexp.escape(extension)
  escaped_replacement = Regexp.escape(replacement)

  # no / in filenames allowed in linux:  https://stackoverflow.com/q/9847288/2062384
  # If there's a / then our filename includes a directory.  We don't want to gsub
  # the directory name because the rename will fail
  dirname = File.dirname(input_file)
  dirs = dirname == '.' ? '' : dirname
  fname = File.basename(input_file)

  retval = fname
             .gsub(/×/, 'x')  # change × to x
             .gsub(/(\s|[.,":?'#;&*\\])/, replacement)
             .gsub(/[()\[\]]/, replacement)
             .gsub(/(\(|\))/, replacement)

  retval = retval.gsub(/#{escaped_replacement}{2,}/, replacement)
  retval = retval.gsub(/#{escaped_replacement}#{escaped_extension}$/, '') unless extension.empty?

  # If there are parent dirs, put them back on, and if there is an extension put that back on
  retval = File.join(dirs, retval) unless dirs.empty?
  retval = "#{retval}.#{extension}" unless extension.empty?
  retval
end

def change_name(old, new)
  FileUtils.mv(old, new)
end

def rename_path(old, new, dry_run: false)
  if old == new
    puts "Old name and new name are the same for '#{old}'.  Not changing"
    return new
  elsif !File.exist?(old)
    puts "Old file name '#{old}' does not exist.  Skipping"
    return old
  elsif File.exist?(new) && old != new
    puts "New file name '#{new}' already exists!  Skipping"
    return old
  end

  action = dry_run ? 'Would change' : 'Changing'
  puts "#{action} '#{old}' to '#{new}'"
  change_name(old, new) unless dry_run
  new
end

def test(input, output)
  processed = sanitized_filename(input)

  if processed == output
    puts "[PASS]: ".green + "Input of " + "'#{input}'".green + " matched " + "'#{output}'".green
  else
    puts "[FAIL]: ".red + "Input of " + "'#{input}'".red + " expected " + "'#{output}'".red + " but got " + "'#{sanitized_filename(input)}'".red
  end
end

def test_recursive_directory
  Dir.mktmpdir do |tmpdir|
    root = File.join(tmpdir, 'dir one')
    FileUtils.mkdir_p(File.join(root, 'sub dir'))
    File.write(File.join(root, 'sub dir', 'file name.txt'), 'test')

    sanitized_root = nil
    capture_stdout do
      sanitized_root = sanitize_directory_tree(root)
    end

    expected_root = File.join(tmpdir, 'dir_one')
    expected_sub = File.join(expected_root, 'sub_dir')
    expected_file = File.join(expected_sub, 'file_name.txt')

    message = "Recursive sanitize of " \
              "#{root.inspect.green} => #{expected_root.inspect.green}"
    if sanitized_root == expected_root && File.directory?(expected_sub) && File.exist?(expected_file)
      puts "[PASS]: ".green + message
    else
      puts "[FAIL]: ".red + message + " failed (actual root #{sanitized_root.inspect})"
    end
  end
end

def test_custom_replacement
  result = sanitized_filename('Hello World.txt', replacement: '-')
  message = "Custom replacement '-' applied"
  if result == 'Hello-World.txt'
    puts "[PASS]: ".green + message
  else
    puts "[FAIL]: ".red + message + " failed (got #{result.inspect})"
  end
end

def test_cli_replacement_option
  argv = ['--replacement', '-']
  options = parse_options(argv)
  message = "CLI replacement option sets '-'"
  if options[:replacement] == '-' && argv.empty?
    puts "[PASS]: ".green + message
  else
    puts "[FAIL]: ".red + message + " failed"
  end
end

def test_recursive_custom_replacement
  Dir.mktmpdir do |tmpdir|
    root = File.join(tmpdir, 'dir one')
    FileUtils.mkdir_p(File.join(root, 'sub dir'))
    File.write(File.join(root, 'sub dir', 'file name.txt'), 'test')

    sanitized_root = nil
    capture_stdout do
      sanitized_root = sanitize_directory_tree(root, replacement: '-')
    end

    expected_root = File.join(tmpdir, 'dir-one')
    expected_sub = File.join(expected_root, 'sub-dir')
    expected_file = File.join(expected_sub, 'file-name.txt')

    message = "Recursive sanitize with '-' replacement"
    if sanitized_root == expected_root && File.directory?(expected_sub) && File.exist?(expected_file)
      puts "[PASS]: ".green + message
    else
      puts "[FAIL]: ".red + message + " failed"
    end
  end
end

def test_recursive_directory_with_nested_content
  Dir.mktmpdir do |tmpdir|
    root = File.join(tmpdir, 'Root Dir')
    child_one = File.join(root, 'Child One')
    child_two = File.join(root, 'Second & Child')
    grand_one = File.join(child_one, 'Grand Child(1)')
    grand_two = File.join(child_two, 'Grand (Final)')

    FileUtils.mkdir_p(grand_one)
    FileUtils.mkdir_p(grand_two)

    files = [
      File.join(root, 'Root File?.txt'),
      File.join(child_one, 'Clip (A).mov'),
      File.join(child_one, 'Clip (B).mov'),
      File.join(grand_one, 'Take #1.wav'),
      File.join(child_two, 'Audio (Draft).wav'),
      File.join(grand_two, 'Mix #2?.wav')
    ]
    files.each { |path| File.write(path, 'test') }

    sanitized_root = nil
    capture_stdout do
      sanitized_root = sanitize_directory_tree(root)
    end

    expected_root = sanitized_filename(root)
    expected_child_one = sanitized_filename(File.join(expected_root, 'Child One'))
    expected_child_two = sanitized_filename(File.join(expected_root, 'Second & Child'))
    expected_grand_one = sanitized_filename(File.join(expected_child_one, 'Grand Child(1)'))
    expected_grand_two = sanitized_filename(File.join(expected_child_two, 'Grand (Final)'))

    expected_files = [
      sanitized_filename(File.join(expected_root, 'Root File?.txt')),
      sanitized_filename(File.join(expected_child_one, 'Clip (A).mov')),
      sanitized_filename(File.join(expected_child_one, 'Clip (B).mov')),
      sanitized_filename(File.join(expected_grand_one, 'Take #1.wav')),
      sanitized_filename(File.join(expected_child_two, 'Audio (Draft).wav')),
      sanitized_filename(File.join(expected_grand_two, 'Mix #2?.wav'))
    ]

    message = "Recursive sanitize handles nested directories and files"
    if sanitized_root == expected_root &&
       [expected_child_one, expected_child_two, expected_grand_one, expected_grand_two].all? { |dir| File.directory?(dir) } &&
       expected_files.all? { |file| File.exist?(file) } &&
       [root, child_one, child_two, grand_one, grand_two].none? { |dir| File.exist?(dir) }
      puts "[PASS]: ".green + message
    else
      puts "[FAIL]: ".red + message + " failed"
    end
  end
end

def test_dry_run
  Dir.mktmpdir do |tmpdir|
    file = File.join(tmpdir, 'file name.txt')
    File.write(file, 'test')
    desired = sanitized_filename(file)

    output = capture_stdout do
      rename_path(file, desired, dry_run: true)
    end

    message = "Dry run sanitize of #{file.inspect.green}"
    if output.include?("Would change") && File.exist?(file) && !File.exist?(desired)
      puts "[PASS]: ".green + message
    else
      puts "[FAIL]: ".red + message + " failed"
    end
  end
end

def test_recursive_dry_run
  Dir.mktmpdir do |tmpdir|
    root = File.join(tmpdir, 'dir one')
    FileUtils.mkdir_p(File.join(root, 'sub dir'))
    File.write(File.join(root, 'sub dir', 'file name.txt'), 'test')

    sanitized_root = nil
    output = capture_stdout do
      sanitized_root = sanitize_directory_tree(root, dry_run: true)
    end

    expected_root = File.join(tmpdir, 'dir_one')
    message = "Recursive dry run sanitize of #{root.inspect.green}"
    if sanitized_root == expected_root && File.exist?(root) &&
       !File.exist?(expected_root) && output.include?("Would change")
      puts "[PASS]: ".green + message
    else
      puts "[FAIL]: ".red + message + " failed"
    end
  end
end

def test_option_terminator
  Dir.mktmpdir do |tmpdir|
    Dir.chdir(tmpdir) do
      file = '-file name.txt'
      File.write(file, 'test')

      argv = ['--dry-run', '--', file.dup]
      options = parse_options(argv)
      sanitized = sanitized_filename(file, replacement: options[:replacement])

      output = capture_stdout do
        rename_path(file, sanitized, dry_run: options[:dry_run])
      end

      message = "Option terminator handles #{file.inspect.green}"
      if options[:dry_run] && options[:replacement] == '_' && argv == [file] &&
         output.include?("Would change") && File.exist?(file) && !File.exist?(sanitized)
        puts "[PASS]: ".green + message
      else
        puts "[FAIL]: ".red + message + " failed"
      end
    end
  end
end

def test_invalid_replacement
  output = capture_stderr do
    begin
      parse_options(['--replacement', '/'])
    rescue SystemExit
      # expected
    end
  end

  message = "Invalid replacement '/' rejected"
  if output.include?("Replacement character '/' is not allowed")
    puts "[PASS]: ".green + message
  else
    puts "[FAIL]: ".red + message + " failed"
  end
end

def run_tests
  require 'colorize'

  test "×", "x"
  test "Hello", "Hello"
  test "hello.wav", "hello.wav"
  test "Hello World", "Hello_World"
  test "Hello.World", "Hello.World"
  test "hello world.wav", "hello_world.wav"
  test "Hello.world.wav", "Hello_world.wav"
  test "hello? + world.wav", "hello_+_world.wav"
  test "Bart_banner_14_5_×_2_5_in.png", "Bart_banner_14_5_x_2_5_in.png"
  test "hello? &&*()#@+ world.wav", "hello_@+_world.wav"
  test "August Gold Q&A Audio.m4a.wav", "August_Gold_Q_A_Audio_m4a.wav"
  test "nested/dir/file name.txt", "nested/dir/file_name.txt"
  test "/absolute/path/Hello World.txt", "/absolute/path/Hello_World.txt"
  test "relative/./path/Hello World.txt", "relative/./path/Hello_World.txt"
  test_recursive_directory
  test_recursive_directory_with_nested_content
  test_custom_replacement
  test_cli_replacement_option
  test_recursive_custom_replacement
  test_dry_run
  test_recursive_dry_run
  test_option_terminator
  test_invalid_replacement
end

def usage(io = $stdout)
  io.puts <<~USAGE
    Usage: sanitize_filename.rb [options] [FILES...]

    Options:
      -t, --test       Run built-in tests and exit
      -r, --recursive  Recursively sanitize directories and their contents
      -n, --dry-run    Show actions without renaming files
      -c, --replacement CHAR
                        Replacement character to use (default: _)
      -h, --help       Show this help message and exit

    Provide one or more files or directories to sanitize their names in-place.
    Use '--' to stop option parsing when filenames begin with '-'.

    Examples:
      # sanitize a single file in the current directory
      sanitize_filename.rb "My File.txt"

      # preview changes without renaming
      sanitize_filename.rb --dry-run "My File.txt"

      # sanitize recursively and use '-' as the separator
      sanitize_filename.rb --recursive --replacement - ~/Downloads

      # sanitize a file whose name starts with a dash
      sanitize_filename.rb -- --weird name.mp3
  USAGE
end

def validate_replacement(char)
  raise ArgumentError, 'Replacement character cannot be empty' if char.nil? || char.empty?
  raise ArgumentError, 'Replacement character must be a single character' unless char.length == 1

  illegal = [File::SEPARATOR, File::ALT_SEPARATOR].compact
  if illegal.include?(char)
    raise ArgumentError, "Replacement character '#{char}' is not allowed"
  end

  char
end

def parse_options(argv)
  options = { run_tests: false, recursive: false, dry_run: false, replacement: '_' }

  parser = OptionParser.new do |opts|
    opts.banner = 'Usage: sanitize_filename.rb [options] [FILES...]'
    opts.on('-t', '--test', 'Run built-in tests and exit') { options[:run_tests] = true }
    opts.on('-r', '--recursive', 'Recursively sanitize directories and their contents') { options[:recursive] = true }
    opts.on('-n', '--dry-run', 'Show actions without renaming files') { options[:dry_run] = true }
    opts.on('-cCHAR', '--replacement CHAR', 'Replacement character to use (default: _)') do |char|
      begin
        options[:replacement] = validate_replacement(char)
      rescue ArgumentError => e
        raise OptionParser::InvalidArgument, e.message
      end
    end
    opts.on('-h', '--help', 'Show this help message and exit') do
      usage
      exit 0
    end
  end

  parser.parse!(argv)
  options
rescue OptionParser::InvalidOption, OptionParser::InvalidArgument => e
  warn e.message
  usage($stderr)
  exit 1
end

def sanitize_directory_tree(path, dry_run: false, replacement: '_')
  unless File.exist?(path)
    puts "Old file name '#{path}' does not exist.  Skipping"
    return path
  end

  unless File.directory?(path) && !File.symlink?(path)
    return rename_path(
      path,
      sanitized_filename(path, replacement: replacement),
      dry_run: dry_run
    )
  end

  Dir.children(path).each do |entry|
    child_path = File.join(path, entry)

    if File.directory?(child_path) && !File.symlink?(child_path)
      sanitize_directory_tree(child_path, dry_run: dry_run, replacement: replacement)
    else
      rename_path(
        child_path,
        sanitized_filename(child_path, replacement: replacement),
        dry_run: dry_run
      )
    end
  end

  rename_path(
    path,
    sanitized_filename(path, replacement: replacement),
    dry_run: dry_run
  )
end

def main(args)
  options = parse_options(args)

  if options[:run_tests]
    run_tests
    exit 0
  end

  targets = args.reject{ |name| %w[. ..].include?(name) }

  if targets.empty?
    usage($stderr)
    exit 1
  end

  targets.each do |target|
    if options[:recursive]
      sanitize_directory_tree(
        target,
        dry_run: options[:dry_run],
        replacement: options[:replacement]
      )
    else
      rename_path(
        target,
        sanitized_filename(target, replacement: options[:replacement]),
        dry_run: options[:dry_run]
      )
    end
  end
end

main ARGV
