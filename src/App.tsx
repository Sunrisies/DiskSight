"use client"

import { useState, useMemo, useCallback } from "react"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Separator } from "@/components/ui/separator"
import { FolderOpen, File, RefreshCw, FolderSearch, Moon, Sun, HardDrive, Clock, Files, Loader2 } from "lucide-react"
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { conversionTime } from 'sunrise-utils'
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
  const [loadingProgress, setLoadingProgress] = useState<string>("")
  const filteredFiles = useMemo(() => {
    console.log(files, 'files')
    let result: FileItem[] = [...files!]
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

  const handleRefresh = () => {
    setIsRefreshing(true)
    setTimeout(() => setIsRefreshing(false), 500)
  }
  const fetchDirectory = useCallback(
    async (path: string) => {
      if (!path) return

      setIsLoading(true)
      setLoadingProgress("正在扫描目录...")

      try {
        // Tauri environment - use actual invoke
        const result = await invoke<DirectoryResult>("get_list_directory", {
          path,
          parallel: parallelProcessing,
        })
        console.log(result, 'result')
        setFiles(() => result.entries)
        setCurrentPath(path)
        setRefreshTime(Number(result.query_time.toFixed(2)))
      } catch (err) {
        console.error("Failed to fetch directory:", err)
      } finally {
        setIsLoading(false)
      }
    },
    [parallelProcessing],
  )

  // 选择文件
  const handleSelectFile = async () => {
    console.log('打开文件')
    const selected = await open({
      directory: true,
      // multiple: true,
    });
    console.log(selected, 'selected')
    await fetchDirectory(selected!)

  }
  return (
    <div className={darkMode ? "dark" : ""}>
      <div className="h-full w-full  overflow-hidden bg-background text-foreground flex flex-col">
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
          <Button variant="ghost" size="icon" onClick={() => setDarkMode(!darkMode)} className="h-7 w-7">
            {darkMode ? <Sun className="h-3.5 w-3.5" /> : <Moon className="h-3.5 w-3.5" />}
          </Button>
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
            <div className="flex items-center gap-3">
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
            <div className="flex items-center gap-3">
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
            </div>
          </div>
        </div>

        {/* Path Navigation */}
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border">
          <span className="text-xs text-muted-foreground shrink-0">当前目录:</span>
          <Input
            value={currentPath}
            onChange={(e) => setCurrentPath(e.target.value)}
            className="h-7 flex-1 font-mono text-xs px-2"
          />
          <Button variant="outline" size="sm" className="h-7 px-2 text-xs gap-1.5 bg-transparent" onClick={handleSelectFile}>
            <FolderSearch className="h-3.5 w-3.5" />
            浏览
          </Button>
          <Button
            variant="default"
            size="sm"
            onClick={handleRefresh}
            className="h-7 px-2 text-xs gap-1.5"
            disabled={isRefreshing}
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
            刷新
          </Button>
        </div>

        {/* File Table - Scrollable */}
        <div className="h-[400px] overflow-auto">
          {isLoading && (
            <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-10">
              <div className="flex flex-col items-center gap-2 text-sm text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" />
                <span>{loadingProgress || "正在加载..."}</span>
              </div>
            </div>
          )}
          <Table>
            <TableHeader className="sticky top-0 bg-muted/80 backdrop-blur-sm">
              <TableRow className="hover:bg-transparent border-border">
                <TableHead className="h-8 text-xs font-semibold w-16">类型</TableHead>
                <TableHead className="h-8 text-xs font-semibold w-16">权限</TableHead>
                <TableHead className="h-8 text-xs font-semibold w-20 text-right">大小</TableHead>
                {showTimeInfo && <TableHead className="h-8 text-xs font-semibold w-32">修改时间</TableHead>}
                <TableHead className="h-8 text-xs font-semibold">路径</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredFiles.map((file, index) => (
                <TableRow
                  key={index}
                  className="group cursor-pointer border-border/50 hover:bg-accent/50 transition-colors"
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
                    {humanReadableSize ? file.size_display : file.size_raw}
                  </TableCell>
                  {showTimeInfo && (
                    <TableCell className="py-1.5 px-3 text-xs text-muted-foreground">{conversionTime(file.created_time.secs_since_epoch)}</TableCell>
                  )}
                  <TableCell className="py-1.5 px-3">
                    <span className="font-mono text-xs group-hover:text-primary transition-colors truncate block max-w-[300px]">
                      {showFullPath ? file.path : file.name}
                    </span>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-1.5 border-t border-border bg-muted/30 text-[10px] text-muted-foreground">
          <span>DiskSight v1.0.1 | 开发: Sunrise</span>
          <span>© 2025 All rights reserved</span>
        </div>
      </div>
    </div>
  )
}
