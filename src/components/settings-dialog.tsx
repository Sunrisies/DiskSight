"use client"

import type React from "react"

import { useState, useEffect, useCallback } from "react"
import { Button } from "@/components/ui/button"
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import { Slider } from "@/components/ui/slider"
import { Separator } from "@/components/ui/separator"
import { Settings, Monitor, Palette, Cog, RotateCcw, Save } from "lucide-react"
import { cn } from "@/lib/utils"
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { moveWindow, Position } from '@tauri-apps/plugin-positioner';
const isTauri = typeof window !== "undefined" && "__TAURI__" in window


const positionConfig: { value: Position; label: string; icon: string }[] = [
    { value: Position.TopLeft, label: "左上", icon: "↖" },
    { value: Position.TopCenter, label: "上中", icon: "↑" },
    { value: Position.TopRight, label: "右上", icon: "↗" },
    { value: Position.LeftCenter, label: "左中", icon: "←" },
    { value: Position.Center, label: "居中", icon: "◎" },
    { value: Position.RightCenter, label: "右中", icon: "→" },
    { value: Position.BottomLeft, label: "左下", icon: "↙" },
    { value: Position.BottomCenter, label: "下中", icon: "↓" },
    { value: Position.BottomRight, label: "右下", icon: "↘" },
]
interface WindowConfig {
    position: Position
    width: number
    height: number
}

interface AppConfig {
    theme: "light" | "dark" | "system"
    alwaysOnTop: boolean
    autoRefresh: boolean
    refreshInterval: number
    defaultPath: string
    showHiddenByDefault: boolean
    parallelByDefault: boolean
    language: string
}

interface SettingsDialogProps {
    open?: boolean
    onOpenChange?: (open: boolean) => void
    darkMode: boolean
    onDarkModeChange: (value: boolean) => void
    defaultPath: string
    onDefaultPathChange: (value: string) => void
    parallelByDefault: boolean
    onParallelByDefaultChange: (value: boolean) => void
    showHiddenByDefault: boolean
    onShowHiddenByDefaultChange: (value: boolean) => void
    trigger?: React.ReactNode
}

export function SettingsDialog({
    open,
    onOpenChange,
    darkMode,
    onDarkModeChange,
    defaultPath,
    onDefaultPathChange,
    parallelByDefault,
    onParallelByDefaultChange,
    showHiddenByDefault,
    onShowHiddenByDefaultChange,
    trigger,
}: SettingsDialogProps) {
    const [windowConfig, setWindowConfig] = useState<WindowConfig>({
        position: Position.Center,
        width: 800,
        height: 600,
    })

    const [appConfig, setAppConfig] = useState<AppConfig>({
        theme: darkMode ? "dark" : "light",
        alwaysOnTop: false,
        autoRefresh: false,
        refreshInterval: 30,
        defaultPath: defaultPath,
        showHiddenByDefault: showHiddenByDefault,
        parallelByDefault: parallelByDefault,
        language: "zh-CN",
    })

    const [isLoading, setIsLoading] = useState(false)

    // Fetch current window position and size from Tauri
    const fetchWindowConfig = useCallback(async () => {
        if (isTauri) {
            try {
                const { getCurrentWindow } = await import("@tauri-apps/api/window")
                const appWindow = getCurrentWindow()
                const size = await appWindow.outerSize()

                setWindowConfig((prev) => ({
                    ...prev,
                    width: size.width,
                    height: size.height,
                }))
            } catch (err) {
                console.error("Failed to get window config:", err)
            }
        }
    }, [])

    useEffect(() => {
        if (open) {
            fetchWindowConfig()
        }
    }, [open, fetchWindowConfig])

    // Apply window configuration
    const applyWindowConfig = async () => {
        try {
            setIsLoading(true)
            const { getCurrentWindow, LogicalSize } = await import("@tauri-apps/api/window")
            const appWindow = getCurrentWindow()
            await appWindow.setSize(new LogicalSize(windowConfig.width, windowConfig.height))
            moveWindow(windowConfig.position)
        } catch (err) {
            console.error("Failed to apply window size:", err)
        } finally {
            setIsLoading(false)
        }
    }

    // Apply app configuration
    const applyAppConfig = () => {
        onDarkModeChange(appConfig.theme === "dark")
        onDefaultPathChange(appConfig.defaultPath)
        onParallelByDefaultChange(appConfig.parallelByDefault)
        onShowHiddenByDefaultChange(appConfig.showHiddenByDefault)
    }

    // Set always on top
    const toggleAlwaysOnTop = async (value: boolean) => {
        setAppConfig((prev) => ({ ...prev, alwaysOnTop: value }))
        if (isTauri) {
            try {
                const { getCurrentWindow } = await import("@tauri-apps/api/window")
                const appWindow = getCurrentWindow()
                await appWindow.setAlwaysOnTop(value)
            } catch (err) {
                console.error("Failed to set always on top:", err)
            }
        }
    }

    // Reset to default
    const resetWindowConfig = () => {
        setWindowConfig({
            position: Position.Center,
            width: 520,
            height: 860,
        })
    }

    const resetAppConfig = () => {
        setAppConfig({
            theme: "dark",
            alwaysOnTop: false,
            autoRefresh: false,
            refreshInterval: 30,
            defaultPath: "",
            showHiddenByDefault: false,
            parallelByDefault: true,
            language: "zh-CN",
        })
    }

    // Save all settings
    const handleSaveAll = async () => {
        setIsLoading(true)
        await applyWindowConfig()
        applyAppConfig()
        setIsLoading(false)
        onOpenChange?.(false)
    }
    const moveToPosition = async (value: string) => {
        const selectedPos = positionConfig.find((pos) => pos.label === value)?.value
        setWindowConfig((prev) => ({ ...prev, position: selectedPos! }))
    }
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            {trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
            <DialogContent className="max-w-[460px] p-0 gap-0">
                <DialogHeader className="px-4 py-3 border-b border-border">
                    <DialogTitle className="flex items-center gap-2 text-base">
                        <Settings className="h-4 w-4" />
                        应用设置
                    </DialogTitle>
                    <DialogDescription className="text-xs">调整窗口位置、大小和其他配置参数</DialogDescription>
                </DialogHeader>

                <Tabs defaultValue="window" className="w-full">
                    <TabsList className="w-full justify-start rounded-none border-b border-border bg-transparent h-9 px-4">
                        <TabsTrigger value="window" className="gap-1.5 text-xs data-[state=active]:bg-muted rounded-sm h-7">
                            <Monitor className="h-3 w-3" />
                            窗口
                        </TabsTrigger>
                        <TabsTrigger value="appearance" className="gap-1.5 text-xs data-[state=active]:bg-muted rounded-sm h-7">
                            <Palette className="h-3 w-3" />
                            外观
                        </TabsTrigger>
                        <TabsTrigger value="general" className="gap-1.5 text-xs data-[state=active]:bg-muted rounded-sm h-7">
                            <Cog className="h-3 w-3" />
                            常规
                        </TabsTrigger>
                    </TabsList>

                    {/* Window Settings Tab */}
                    <TabsContent value="window" className="m-0 p-4 space-y-4">
                        <div className="space-y-3">
                            <div className="flex items-center justify-between">
                                <div className="flex gap-3 items-center">
                                    <h4 className="text-sm font-medium">窗口位置</h4>
                                    <p className="text-[10px] text-muted-foreground text-center">点击选择窗口在屏幕上的位置</p>
                                </div>


                                <Button variant="ghost" size="sm" className="h-7 text-xs gap-1" onClick={resetWindowConfig}>
                                    <RotateCcw className="h-3 w-3" />
                                    重置
                                </Button>
                            </div>
                            <Select value={positionConfig.find((pos) => pos.value === windowConfig.position)?.label} onValueChange={(value) => moveToPosition(value)}
                            >
                                <SelectTrigger className="w-[180px]">
                                    <SelectValue placeholder="居中" />
                                </SelectTrigger>
                                <SelectContent>
                                    {positionConfig.map((pos) => (
                                        <SelectItem
                                            value={pos.label}
                                            key={pos.value}
                                            className={cn(
                                                windowConfig.position === pos.value && "ring-2 ring-primary ring-offset-1 ring-offset-background",
                                            )}
                                        >{pos.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        <Separator />

                        <div className="space-y-3">
                            <h4 className="text-sm font-medium">窗口大小</h4>
                            <div className="grid grid-cols-2 gap-3">
                                <div className="space-y-1.5">
                                    <Label htmlFor="window-width" className="text-xs text-muted-foreground">
                                        宽度 (像素)
                                    </Label>
                                    <Input
                                        id="window-width"
                                        type="number"
                                        value={windowConfig.width}
                                        onChange={(e) =>
                                            setWindowConfig((prev) => ({ ...prev, width: Number.parseInt(e.target.value) || 520 }))
                                        }
                                        className="h-8 text-xs"
                                    />
                                </div>
                                <div className="space-y-1.5">
                                    <Label htmlFor="window-height" className="text-xs text-muted-foreground">
                                        高度 (像素)
                                    </Label>
                                    <Input
                                        id="window-height"
                                        type="number"
                                        value={windowConfig.height}
                                        onChange={(e) =>
                                            setWindowConfig((prev) => ({ ...prev, height: Number.parseInt(e.target.value) || 860 }))
                                        }
                                        className="h-8 text-xs"
                                    />
                                </div>
                            </div>
                        </div>

                        <Separator />

                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label className="text-sm font-medium">窗口置顶</Label>
                                <p className="text-xs text-muted-foreground">始终保持窗口在最前面</p>
                            </div>
                            <Switch checked={appConfig.alwaysOnTop} onCheckedChange={toggleAlwaysOnTop} />
                        </div>

                        <Button
                            variant="outline"
                            size="sm"
                            className="w-full h-8 text-xs bg-transparent"
                            onClick={applyWindowConfig}
                            disabled={isLoading}
                        >
                            应用窗口设置
                        </Button>
                    </TabsContent>

                    {/* Appearance Settings Tab */}
                    <TabsContent value="appearance" className="m-0 p-4 space-y-4">
                        <div className="space-y-3">
                            <h4 className="text-sm font-medium">主题</h4>
                            <div className="grid grid-cols-3 gap-2">
                                {(["light", "dark", "system"] as const).map((theme) => (
                                    <Button
                                        key={theme}
                                        variant={appConfig.theme === theme ? "default" : "outline"}
                                        size="sm"
                                        className="h-8 text-xs"
                                        onClick={() => setAppConfig((prev) => ({ ...prev, theme }))}
                                    >
                                        {theme === "light" && "浅色"}
                                        {theme === "dark" && "深色"}
                                        {theme === "system" && "跟随系统"}
                                    </Button>
                                ))}
                            </div>
                        </div>

                        <Separator />

                        <div className="space-y-3">
                            <h4 className="text-sm font-medium">语言</h4>
                            <div className="grid grid-cols-2 gap-2">
                                {[
                                    { value: "zh-CN", label: "简体中文" },
                                    { value: "en-US", label: "English" },
                                ].map((lang) => (
                                    <Button
                                        key={lang.value}
                                        variant={appConfig.language === lang.value ? "default" : "outline"}
                                        size="sm"
                                        className="h-8 text-xs"
                                        onClick={() => setAppConfig((prev) => ({ ...prev, language: lang.value }))}
                                    >
                                        {lang.label}
                                    </Button>
                                ))}
                            </div>
                        </div>
                    </TabsContent>

                    {/* General Settings Tab */}
                    <TabsContent value="general" className="m-0 p-4 space-y-4">
                        <div className="space-y-3">
                            <div className="flex items-center justify-between">
                                <h4 className="text-sm font-medium">默认设置</h4>
                                <Button variant="ghost" size="sm" className="h-7 text-xs gap-1" onClick={resetAppConfig}>
                                    <RotateCcw className="h-3 w-3" />
                                    重置
                                </Button>
                            </div>

                            <div className="space-y-1.5">
                                <Label htmlFor="default-path" className="text-xs text-muted-foreground">
                                    默认扫描路径
                                </Label>
                                <Input
                                    id="default-path"
                                    value={appConfig.defaultPath}
                                    onChange={(e) => setAppConfig((prev) => ({ ...prev, defaultPath: e.target.value }))}
                                    placeholder="例如: D:\project"
                                    className="h-8 text-xs font-mono"
                                />
                            </div>
                        </div>

                        <Separator />

                        <div className="space-y-3">
                            <h4 className="text-sm font-medium">默认选项</h4>

                            <div className="flex items-center justify-between">
                                <div className="space-y-0.5">
                                    <Label className="text-sm">显示隐藏文件</Label>
                                    <p className="text-xs text-muted-foreground">启动时默认显示隐藏文件</p>
                                </div>
                                <Switch
                                    checked={appConfig.showHiddenByDefault}
                                    onCheckedChange={(value) => setAppConfig((prev) => ({ ...prev, showHiddenByDefault: value }))}
                                />
                            </div>

                            <div className="flex items-center justify-between">
                                <div className="space-y-0.5">
                                    <Label className="text-sm">并行处理</Label>
                                    <p className="text-xs text-muted-foreground">启动时默认开启并行处理</p>
                                </div>
                                <Switch
                                    checked={appConfig.parallelByDefault}
                                    onCheckedChange={(value) => setAppConfig((prev) => ({ ...prev, parallelByDefault: value }))}
                                />
                            </div>
                        </div>

                        <Separator />

                        <div className="space-y-3">
                            <div className="flex items-center justify-between">
                                <div className="space-y-0.5">
                                    <Label className="text-sm font-medium">自动刷新</Label>
                                    <p className="text-xs text-muted-foreground">定时自动刷新目录列表</p>
                                </div>
                                <Switch
                                    checked={appConfig.autoRefresh}
                                    onCheckedChange={(value) => setAppConfig((prev) => ({ ...prev, autoRefresh: value }))}
                                />
                            </div>

                            {appConfig.autoRefresh && (
                                <div className="space-y-2">
                                    <div className="flex items-center justify-between">
                                        <Label className="text-xs text-muted-foreground">刷新间隔 (秒)</Label>
                                        <span className="text-xs font-medium">{appConfig.refreshInterval}s</span>
                                    </div>
                                    <Slider
                                        value={[appConfig.refreshInterval]}
                                        onValueChange={([value]) => setAppConfig((prev) => ({ ...prev, refreshInterval: value }))}
                                        min={5}
                                        max={300}
                                        step={5}
                                        className="w-full"
                                    />
                                </div>
                            )}
                        </div>
                    </TabsContent>
                </Tabs>

                <DialogFooter className="px-4 py-3 border-t border-border">
                    <Button
                        variant="outline"
                        size="sm"
                        className="h-8 text-xs bg-transparent"
                        onClick={() => onOpenChange?.(false)}
                    >
                        取消
                    </Button>
                    <Button size="sm" className="h-8 text-xs gap-1.5" onClick={handleSaveAll} disabled={isLoading}>
                        <Save className="h-3 w-3" />
                        保存设置
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
