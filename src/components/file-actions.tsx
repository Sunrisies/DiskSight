import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
import {
    sendNotification,
} from '@tauri-apps/plugin-notification';

interface FileActionsProps {
    filePath: string;
    onRefresh: () => void; // 用于刷新文件列表的回调函数
}

export const FileActions: React.FC<FileActionsProps> = ({ filePath, onRefresh }) => {
    const [isDeleting, setIsDeleting] = useState(false);

    const handleDelete = async () => {
        // 显示确认对话框
        const confirmed = await confirm(
            '确定要删除这个文件吗？此操作不可撤销。',
            {
                title: '确认删除',
            }
        );

        if (!confirmed) return;

        setIsDeleting(true);
        try {
            // 调用后端删除文件的命令
            await invoke('delete_file', { path: filePath, force: false });

            // 显示成功通知
            await sendNotification({
                title: '删除成功',
                body: '文件已被成功删除'
            });

            // 刷新文件列表
            onRefresh();
        } catch (error) {
            console.error('删除文件失败:', error);
            await sendNotification({
                title: '删除失败',
                body: '文件删除失败，请检查权限'
            });
        } finally {
            setIsDeleting(false);
        }
    };

    return (
        <Button
            className="cursor-pointer"
            size="sm"
            onClick={handleDelete}
            disabled={isDeleting}
            variant={isDeleting ? "secondary" : "destructive"}
        >
            {isDeleting ? '删除中...' : '删除'}
        </Button>
    );
};
