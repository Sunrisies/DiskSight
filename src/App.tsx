"use client"

import { useState, useMemo, useCallback, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Separator } from "@/components/ui/separator"
import { FolderOpen, File, RefreshCw, FolderSearch, Moon, Sun, HardDrive, Settings, Clock, Files, Loader2, X } from "lucide-react"
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { conversionTime } from 'sunrise-utils'
import { cn } from "./lib/utils"
import { SettingsDialog } from "@/components/settings-dialog"
import { FileActions } from '@/components/file-actions'
interface ICreatedTime {
  nanos_since_epoch: number
  secs_since_epoch: number
}

interface FileItem {
  file_type: string
  permissions: string
  size_raw: number
  size_display: string
  path: string
  name: string
  created_time: ICreatedTime
}

interface DirectoryResult {
  entries: FileItem[],
  query_time: number
}

// 进度事件接口
interface ProgressEvent {
  current_path: string
  current_file: string
  status: string
}

function formatBytes(bytes: number, humanReadable: boolean): string {
  if (!humanReadable) return `${bytes}B`
  if (bytes === 0) return "0B"
  const k = 1024
  const sizes = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(1))}${sizes[i]}`
}

export default function DiskSight() {
  const [darkMode, setDarkMode] = useState(false)
  const [humanReadableSize, setHumanReadableSize] = useState(true)
  const [showHiddenFiles, setShowHiddenFiles] = useState(false)
  const [showTimeInfo, setShowTimeInfo] = useState(false)
  const [showFullPath, setShowFullPath] = useState(false)
  const [parallelProcessing, setParallelProcessing] = useState(true)
  const [sortBySize, setSortBySize] = useState(true)
  const [files, setFiles] = useState<FileItem[]>([])
  const [currentPath, setCurrentPath] = useState("")
  const [refreshTime, setRefreshTime] = useState(0)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [scanProgress, setScanProgress] = useState<ProgressEvent | null>(null)
  const [error, setError] = useState<string | null>(null)
  // 是否开启文件扫描详情
  const [showScanDetails, setShowScanDetails] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)

  // 新增状态：历史记录和文件详情
  const [history, setHistory] = useState<string[]>([])
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [showFileDetail, setShowFileDetail] = useState(false)
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null)
  // 事件监听
  useEffect(() => {
    let unlistenStarted: UnlistenFn | undefined;
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenCompleted: UnlistenFn | undefined;
    let unlistenError: UnlistenFn | undefined;

    const setupListeners = async () => {
      try {
        unlistenStarted = await listen('scan-started', () => {
          setIsLoading(true);
          setError(null);
          setScanProgress(null);
        });

        unlistenProgress = await listen('scan-progress', (event: { payload: ProgressEvent }) => {
          setScanProgress(event.payload);
        });

        unlistenCompleted = await listen('scan-completed', () => {
          setIsLoading(false);
          setScanProgress(null);
        });

        unlistenError = await listen('scan-error', (event) => {
          setIsLoading(false);
          setScanProgress(null);
          setError(event.payload as string);
        });
      } catch (error) {
        console.error('Failed to setup event listeners:', error);
      }
    };

    setupListeners();

    return () => {
      unlistenStarted?.();
      unlistenProgress?.();
      unlistenCompleted?.();
      unlistenError?.();
    };
  }, []);

  const filteredFiles = useMemo(() => {
    let result: FileItem[] = [...files]
    if (!showHiddenFiles) {
      result = result.filter((f) => !f.name.startsWith("."))
    }
    if (sortBySize) {
      result.sort((a, b) => b.size_raw - a.size_raw)
    }
    return result.length > 0 ? result : []
  }, [files, showHiddenFiles, sortBySize])

  const totalSize = useMemo(() => {
    return filteredFiles.reduce((acc, f) => acc + f.size_raw, 0)
  }, [filteredFiles])

  const handleRefresh = useCallback(() => {
    console.log("Refreshing...", showScanDetails)
    if (currentPath) {
      setIsRefreshing(true)
      fetchDirectory(currentPath, showScanDetails)
      setTimeout(() => setIsRefreshing(false), 500)
    }
  }, [currentPath, showScanDetails])

  const fetchDirectory = useCallback(async (path: string, showDetails: boolean = false) => {
    if (!path) return

    setIsLoading(true)
    setError(null)
    let result: DirectoryResult
    console.log("Fetching directory:", path, showScanDetails)
    try {
      if (showDetails) {
        result = await invoke<DirectoryResult>("get_list_directory", {
          path,
        })
      } else {
        result = await invoke<DirectoryResult>("calculate_dir_size_simple_fast", {
          path,
        })
        setIsLoading(false);
        setScanProgress(null);
      }

      setFiles(result.entries)
      setCurrentPath(path)
      console.log("Directory fetched:", result)
      setRefreshTime(Number(result.query_time.toFixed(2)))
    } catch (err) {
      console.error("Failed to fetch directory:", err)
      setError(err instanceof Error ? err.message : "获取目录失败")
    }
  }, [parallelProcessing, humanReadableSize, showHiddenFiles, sortBySize, showTimeInfo, showFullPath])

  // 选择目录
  const handleSelectFile = async () => {
    const selected = await open({
      directory: true,
    })

    if (selected) {
      // 如果是第一次选择目录，初始化历史记录
      if (history.length === 0) {
        setHistory([selected])
        setHistoryIndex(0)
      }
      await fetchDirectory(selected)
    }
  }

  // 取消扫描
  const handleCancelScan = () => {
    setIsLoading(false)
    setScanProgress(null)
    // 注意：这里需要后端支持取消操作，目前只是前端状态重置
  }

  // 获取状态显示文本
  const getStatusText = (status: string) => {
    const statusMap: Record<string, string> = {
      'processing': '处理中',
      'processing_batch': '批量处理中',
      'calculating_directory_size': '计算目录大小',
      'directory_calculation_completed': '目录计算完成',
      'completed': '完成',
      'searching_in_directory': '在目录中搜索',
      'checking_file': '检查文件',
      'calculating_matching_directory': '计算匹配目录',
      'matching_directory_completed': '匹配目录完成',
      'processing_file': '处理文件'
    }
    return statusMap[status] || status
  }

  // 处理表格行点击
  const handleTableRowClick = useCallback((file: FileItem) => {
    if (file.file_type === "d") {
      // 如果是目录，进入该目录
      const newPath = file.path

      // 更新历史记录
      const newHistory = history.slice(0, historyIndex + 1)
      newHistory.push(newPath)
      setHistory(newHistory)
      setHistoryIndex(newHistory.length - 1)

      // 加载新目录
      fetchDirectory(newPath)
    } else {
      // 如果是文件，显示文件详情
      setSelectedFile(file)
      setShowFileDetail(true)
    }
  }, [history, historyIndex, fetchDirectory])

  // 回退功能
  const navigateBack = useCallback(() => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1
      const prevPath = history[newIndex]
      setHistoryIndex(newIndex)
      fetchDirectory(prevPath)
    }
  }, [history, historyIndex, fetchDirectory])

  // 前进功能（如果需要）
  const navigateForward = useCallback(() => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1
      const nextPath = history[newIndex]
      setHistoryIndex(newIndex)
      fetchDirectory(nextPath)
    }
  }, [history, historyIndex, fetchDirectory])

  // 是否可以回退
  const canGoBack = historyIndex > 0
  // 是否可以前进
  const canGoForward = historyIndex < history.length - 1

  // 关闭文件详情弹窗
  const closeFileDetail = () => {
    setShowFileDetail(false)
    setSelectedFile(null)
  }

  return (
    <div className={cn("h-full", darkMode ? "dark" : "")}>
      <div className="h-full w-full overflow-hidden bg-background text-foreground flex flex-col">
        {/* Header Bar */}
        <div className="flex items-center justify-between px-4 py-2.5 border-b border-border bg-card">
          <div className="flex items-center gap-2.5">
            <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-primary">
              <HardDrive className="h-4 w-4 text-primary-foreground" />
            </div>
            <div>
              <h1 className="text-sm font-semibold leading-tight">目录文件大小查看器</h1>
              <p className="text-[10px] text-muted-foreground">DiskSight v1.0</p>
            </div>
          </div>
          <div className="flex items-center gap-2.5">
            <Button variant="ghost" size="icon" onClick={() => setSettingsOpen(true)} className="h-7 w-7">
              <Settings className="h-3.5 w-3.5" />
            </Button>
            <Button variant="ghost" size="icon" onClick={() => setDarkMode(!darkMode)} className="h-7 w-7">
              {darkMode ? <Sun className="h-3.5 w-3.5" /> : <Moon className="h-3.5 w-3.5" />}
            </Button>
          </div>
        </div>


        {/* Stats Row */}
        <div className="flex items-center gap-4 px-4 py-2 bg-muted/30 border-b border-border text-xs">
          <div className="flex items-center gap-1.5">
            <Files className="h-3.5 w-3.5 text-primary" />
            <span className="text-muted-foreground">总数量:</span>
            <span className="font-semibold">{filteredFiles.length}</span>
          </div>
          <Separator orientation="vertical" className="h-4" />
          <div className="flex items-center gap-1.5">
            <HardDrive className="h-3.5 w-3.5 text-chart-2" />
            <span className="text-muted-foreground">总大小:</span>
            <span className="font-semibold">{formatBytes(totalSize, humanReadableSize)}</span>
          </div>
          <Separator orientation="vertical" className="h-4" />
          <div className="flex items-center gap-1.5">
            <Clock className="h-3.5 w-3.5 text-chart-3" />
            <span className="text-muted-foreground">耗时:</span>
            <span className="font-semibold">{refreshTime}s</span>
          </div>
        </div>

        {/* Options Panel */}
        <div className="px-4 py-2.5 border-b border-border bg-card/50">
          <div className="flex flex-wrap gap-x-6 gap-y-2 text-xs">
            {/* Display Format */}
            <div className="flex items-center gap-3">
              <span className="font-semibold text-muted-foreground uppercase text-[10px] tracking-wider">显示格式</span>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={humanReadableSize}
                  onCheckedChange={(checked) => setHumanReadableSize(checked as boolean)}
                  className="h-3.5 w-3.5"
                />
                <span>人性化大小</span>
              </label>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={showHiddenFiles}
                  onCheckedChange={(checked) => setShowHiddenFiles(checked as boolean)}
                  className="h-3.5 w-3.5"
                />
                <span>隐藏文件</span>
              </label>
            </div>

            {/* Display Content */}
            <div className="flex items-center gap-2">
              <span className="font-semibold text-muted-foreground uppercase text-[10px] tracking-wider">显示内容</span>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={showTimeInfo}
                  onCheckedChange={(checked) => setShowTimeInfo(checked as boolean)}
                  className="h-3.5 w-3.5"
                />
                <span>时间信息</span>
              </label>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={showFullPath}
                  onCheckedChange={(checked) => setShowFullPath(checked as boolean)}
                  className="h-3.5 w-3.5"
                />
                <span>完整路径</span>
              </label>
            </div>

            {/* Processing Options */}
            <div className="flex items-center gap-2">
              <span className="font-semibold text-muted-foreground uppercase text-[10px] tracking-wider">处理选项</span>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={parallelProcessing}
                  onCheckedChange={(checked) => setParallelProcessing(checked as boolean)}
                  className="h-3.5 w-3.5"
                />
                <span>并行处理</span>
              </label>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={sortBySize}
                  onCheckedChange={(checked) => setSortBySize(checked as boolean)}
                  className="h-3.5 w-3.5"
                />
                <span>大小排序</span>
              </label>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <Checkbox
                  checked={showScanDetails}
                  onCheckedChange={async (checked) => {
                    setShowScanDetails(checked as boolean)

                  }}
                  className="h-3.5 w-3.5"
                />
                <span>显示扫描详情</span>
              </label>

            </div>
          </div>
        </div>

        {/* Path Navigation */}
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border">
          <span className="text-xs text-muted-foreground shrink-0">当前目录:</span>
          <div className="flex items-center gap-1 flex-1">
            <Button
              variant="outline"
              size="sm"
              className="h-7 px-2 text-xs gap-1.5"
              onClick={navigateBack}
              disabled={!canGoBack || isLoading}
              title="回退"
            >
              ←
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-7 px-2 text-xs gap-1.5"
              onClick={navigateForward}
              disabled={!canGoForward || isLoading}
              title="前进"
            >
              →
            </Button>
            <Input
              value={currentPath}
              onChange={(e) => setCurrentPath(e.target.value)}
              onKeyPress={(e) => {
                if (e.key === 'Enter') {
                  fetchDirectory(currentPath)
                }
              }}
              className="h-7 flex-1 font-mono text-xs px-2"
              placeholder="输入目录路径或点击浏览选择"
            />
          </div>
          <Button
            variant="outline"
            size="sm"
            className="h-7 px-2 text-xs gap-1.5 bg-transparent"
            onClick={handleSelectFile}
            disabled={isLoading}
          >
            <FolderSearch className="h-3.5 w-3.5" />
            浏览
          </Button>
          <Button
            variant="default"
            size="sm"
            onClick={handleRefresh}
            className="h-7 px-2 text-xs gap-1.5"
            disabled={isRefreshing || isLoading || !currentPath}
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
            刷新
          </Button>
        </div>

        {/* 错误提示 */}
        {error && (
          <div className="mx-4 mt-2 p-3 bg-destructive/10 border border-destructive/20 rounded-lg">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 text-destructive text-sm">
                <span>错误:</span>
                <span>{error}</span>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setError(null)}
                className="h-6 w-6 p-0"
              >
                <X className="h-3 w-3" />
              </Button>
            </div>
          </div>
        )}

        {/* 加载遮罩和进度显示 */}
        {isLoading && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-10">
            <div className="bg-card p-6 rounded-lg border shadow-lg max-w-md w-full mx-4">
              <div className="flex items-center gap-3 mb-4">
                <Loader2 className="h-6 w-6 animate-spin text-primary" />
                <div className="flex-1">
                  <h3 className="font-semibold text-sm mb-2">扫描目录中...</h3>

                  {scanProgress && (
                    <div className="space-y-2 text-xs">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">状态:</span>
                        <span className="font-medium capitalize">{getStatusText(scanProgress.status)}</span>
                      </div>

                      <div className="flex justify-between">
                        <span className="text-muted-foreground">当前目录:</span>
                        <span
                          className="font-medium truncate ml-2 max-w-[200px]"
                          title={scanProgress.current_path}
                        >
                          {scanProgress.current_path.split(/[\\/]/).pop() || scanProgress.current_path}
                        </span>
                      </div>

                      {scanProgress.current_file && (
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">当前文件:</span>
                          <span
                            className="font-medium truncate ml-2 max-w-[200px]"
                            title={scanProgress.current_file}
                          >
                            {scanProgress.current_file.split(/[\\/]/).pop() || scanProgress.current_file}
                          </span>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </div>

              <div className="flex justify-end">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleCancelScan}
                >
                  取消
                </Button>
              </div>
            </div>
          </div>
        )}

        {/* File Table - Scrollable */}
        <div className="flex-1 overflow-auto relative">
          {!isLoading && filteredFiles.length === 0 && (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="text-center text-muted-foreground">
                <FolderSearch className="h-12 w-12 mx-auto mb-2 opacity-50" />
                <p>请选择目录开始浏览</p>
              </div>
            </div>
          )}

          {filteredFiles.length > 0 && (
            <Table >
              <TableHeader className="sticky top-0 bg-muted/80 backdrop-blur-sm">
                <TableRow className="hover:bg-transparent border-border">
                  <TableHead className="h-8 text-xs font-semibold w-16">类型</TableHead>
                  <TableHead className="h-8 text-xs font-semibold w-16">权限</TableHead>
                  <TableHead className="h-8 text-xs font-semibold w-20 text-right">大小</TableHead>
                  {showTimeInfo && <TableHead className="h-8 text-xs font-semibold w-32">修改时间</TableHead>}
                  <TableHead className="h-8 text-xs font-semibold">路径</TableHead>
                  <TableHead className="h-8 text-xs font-semibold">操作</TableHead>

                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredFiles.map((file, index) => (
                  <TableRow
                    key={index}
                    className="group cursor-pointer border-border/50 hover:bg-accent/50 transition-colors"
                    onClick={() => handleTableRowClick(file)}
                  >
                    <TableCell className="py-1.5 px-3">
                      {file.file_type === "d" ? (
                        <Badge
                          variant="outline"
                          className="h-5 gap-1 px-1.5 text-[10px] border-chart-3/50 bg-chart-3/10 text-chart-3 font-medium"
                        >
                          <FolderOpen className="h-2.5 w-2.5" />d
                        </Badge>
                      ) : (
                        <Badge
                          variant="outline"
                          className="h-5 gap-1 px-1.5 text-[10px] border-primary/50 bg-primary/10 text-primary font-medium"
                        >
                          <File className="h-2.5 w-2.5" />-
                        </Badge>
                      )}
                    </TableCell>
                    <TableCell className="py-1.5 px-3">
                      <code className="text-[10px] font-mono text-muted-foreground">{file.permissions}</code>
                    </TableCell>
                    <TableCell className="py-1.5 px-3 text-right font-mono text-xs tabular-nums">
                      {humanReadableSize ? file.size_display : formatBytes(file.size_raw, false)}
                    </TableCell>
                    {showTimeInfo && (
                      <TableCell className="py-1.5 px-3 text-xs text-muted-foreground">
                        {conversionTime(file.created_time.secs_since_epoch)}
                      </TableCell>
                    )}
                    <TableCell className="py-1.5 px-3">
                      <span
                        className="font-mono text-xs group-hover:text-primary transition-colors truncate block max-w-[300px]"
                        title={showFullPath ? file.path : file.name}
                      >
                        {showFullPath ? file.path : file.name}
                      </span>
                    </TableCell>
                    <TableCell className="py-1.5 px-3 w-10">
                      <div onClick={(e) => e.stopPropagation()}>
                        <FileActions filePath={file.path} onRefresh={handleRefresh}></FileActions>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-1.5 border-t border-border bg-muted/30 text-[10px] text-muted-foreground">
          <span>DiskSight v1.0.1 | 开发: Sunrise</span>
          <span>© 2025 All rights reserved</span>
        </div>
      </div>
      {/* Settings Dialog */}
      <SettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        darkMode={darkMode}
        onDarkModeChange={setDarkMode}
        defaultPath={currentPath}
        onDefaultPathChange={setCurrentPath}
        parallelByDefault={parallelProcessing}
        onParallelByDefaultChange={setParallelProcessing}
        showHiddenByDefault={showHiddenFiles}
        onShowHiddenByDefaultChange={setShowHiddenFiles}
      />

      {/* 文件详情弹窗 */}
      {showFileDetail && selectedFile && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50" onClick={closeFileDetail}>
          <div className="bg-card rounded-lg border shadow-lg max-w-md w-full mx-4 p-6" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold text-sm">文件详情</h3>
              <Button variant="ghost" size="sm" onClick={closeFileDetail} className="h-6 w-6 p-0">
                <X className="h-4 w-4" />
              </Button>
            </div>

            <div className="space-y-3 text-xs">
              <div className="flex justify-between">
                <span className="text-muted-foreground">名称:</span>
                <span className="font-medium">{selectedFile.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">类型:</span>
                <span className="font-medium">
                  {selectedFile.file_type === "d" ? "目录" : "文件"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">路径:</span>
                <span className="font-medium font-mono break-all">{selectedFile.path}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">大小:</span>
                <span className="font-medium font-mono">
                  {humanReadableSize ? selectedFile.size_display : formatBytes(selectedFile.size_raw, false)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">权限:</span>
                <span className="font-medium font-mono">{selectedFile.permissions}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">创建时间:</span>
                <span className="font-medium">
                  {conversionTime(selectedFile.created_time.secs_since_epoch)}
                </span>
              </div>
            </div>

            <div className="mt-6 flex justify-end gap-2">
              <Button variant="outline" size="sm" onClick={closeFileDetail}>
                关闭
              </Button>
              <Button size="sm" onClick={() => {
                // 复制路径到剪贴板
                navigator.clipboard.writeText(selectedFile.path)
                closeFileDetail()
              }}>
                复制路径
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
