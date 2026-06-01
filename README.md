# replace-files

一个 CLI 工具，在目标目录中递归查找与参考文件匹配（按文件名、文件大小和内容哈希）的文件，并将其替换为新文件。

## 使用场景

当一个项目中散布着许多相同的文件副本（如图标、占位图片或模板文件），需要将它们全部替换为更新后的版本时，此工具非常有用。它使用三步匹配机制确保只替换正确的文件：

1. **文件名匹配** – 文件名必须完全相同
2. **大小匹配** – 文件大小必须完全相同（快速过滤）
3. **SHA-256 哈希匹配** – 文件内容必须完全相同（最终验证）

## 安装

### 前置条件

- [Rust](https://www.rust-lang.org/tools/install)（2024 edition）

### 从源码编译

```bash
git clone https://github.com/233lol/replace-files.git
cd replace-files
cargo build --release
```

编译后的二进制文件位于 `target/release/replace-files`。你可以将它复制到 `PATH` 中的任意目录，例如：

```bash
cp target/release/replace-files ~/.cargo/bin/
```

## 使用方法

```
replace-files <旧文件> <新文件> <目标目录> [--dry-run]
```

### 参数

| 参数         | 说明                                            |
|-------------|------------------------------------------------|
| `OLD_FILE`  | 参考文件路径，用于匹配需要替换的文件                |
| `NEW_FILE`  | 替换文件路径                                     |
| `TARGET_DIR` | 要递归扫描的目标目录                              |

### 选项

| 选项        | 说明                                        |
|-------------|---------------------------------------------|
| `--dry-run` | 仅列出会被替换的文件，不执行实际替换操作        |

### 示例

#### 将所有匹配 `icon.png` 的文件替换为新图标

```bash
replace-files old-icon.png new-icon.png ./project-directory
```

#### 预览将要被替换的文件（不实际修改）

```bash
replace-files old-icon.png new-icon.png ./project-directory --dry-run
```

## 工作原理

1. 验证 `OLD_FILE`、`NEW_FILE` 和 `TARGET_DIR` 是否存在且类型正确（文件或目录）。
2. 计算 `OLD_FILE` 的 SHA-256 哈希值并记录其文件大小。
3. 递归遍历目标目录。
4. 对每个普通文件：
   - 跳过 `NEW_FILE` 本身（避免替换源文件）。
   - 检查文件名是否与 `OLD_FILE` 的文件名相同。
   - 检查文件大小是否与 `OLD_FILE` 相同。
   - 计算 SHA-256 哈希并与 `OLD_FILE` 的哈希值比较。
5. 如果三个条件全部匹配，则将该文件替换为 `NEW_FILE` 的副本。

## 输出示例

正常替换模式输出：
```
Replaced: ./project-directory/subdir/icon.png
Replaced: ./project-directory/deep/nested/icon.png

Done. 2 file(s) replaced (out of 2 matching).
```

Dry-run 模式输出：
```
[DRY RUN] Would replace: ./project-directory/subdir/icon.png;Size: 12345
[DRY RUN] Would replace: ./project-directory/deep/nested/icon.png;Size: 12345

Dry run complete. 2 file(s) would be replaced.
```

## 许可证

本项目使用仓库中 [LICENSE](LICENSE) 文件规定的许可证。