import { useState } from 'react'
import { Download, Image, FileText, Printer } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from '@/components/ui/dropdown-menu'
import { EXPORT_CONFIG } from '@/lib/chartTheme'

interface ChartExportProps {
  chartRef: React.RefObject<any>
  title?: string
  className?: string
  size?: 'sm' | 'md' | 'lg'
}

export function ChartExport({ chartRef, title = 'chart', className, size = 'md' }: ChartExportProps) {
  const [isExporting, setIsExporting] = useState(false)

  // 导出为PNG
  const exportToPNG = async () => {
    if (!chartRef.current) return

    setIsExporting(true)
    try {
      const canvas = chartRef.current.canvas || chartRef.current.querySelector('canvas')
      if (canvas) {
        const link = document.createElement('a')
        link.download = `${title}-${new Date().toISOString().split('T')[0]}.png`
        link.href = canvas.toDataURL('image/png', 1.0)
        link.click()
      } else {
        // 对于SVG图表，使用html2canvas
        const { default: html2canvas } = await import('html2canvas')
        const canvas = await html2canvas(chartRef.current, {
          backgroundColor: EXPORT_CONFIG.png.backgroundColor,
          scale: EXPORT_CONFIG.png.pixelRatio,
          useCORS: true,
          allowTaint: true,
        })
        
        const link = document.createElement('a')
        link.download = `${title}-${new Date().toISOString().split('T')[0]}.png`
        link.href = canvas.toDataURL('image/png', 1.0)
        link.click()
      }
    } catch (error) {
      console.error('PNG导出失败:', error)
    } finally {
      setIsExporting(false)
    }
  }

  // 导出为SVG
  const exportToSVG = async () => {
    if (!chartRef.current) return

    setIsExporting(true)
    try {
      const svgElement = chartRef.current.querySelector('svg')
      if (svgElement) {
        const svgData = new XMLSerializer().serializeToString(svgElement)
        const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' })
        const svgUrl = URL.createObjectURL(svgBlob)
        
        const link = document.createElement('a')
        link.download = `${title}-${new Date().toISOString().split('T')[0]}.svg`
        link.href = svgUrl
        link.click()
        
        URL.revokeObjectURL(svgUrl)
      } else {
        throw new Error('未找到SVG元素')
      }
    } catch (error) {
      console.error('SVG导出失败:', error)
    } finally {
      setIsExporting(false)
    }
  }

  // 导出为PDF
  const exportToPDF = async () => {
    if (!chartRef.current) return

    setIsExporting(true)
    try {
      const { default: jsPDF } = await import('jspdf')
      const { default: html2canvas } = await import('html2canvas')
      
      const canvas = await html2canvas(chartRef.current, {
        backgroundColor: '#ffffff',
        scale: 2,
        useCORS: true,
        allowTaint: true,
      })
      
      const imgData = canvas.toDataURL('image/png')
      const pdf = new jsPDF({
        orientation: EXPORT_CONFIG.pdf.orientation,
        unit: 'mm',
        format: EXPORT_CONFIG.pdf.format,
      })
      
      const pdfWidth = pdf.internal.pageSize.getWidth()
      const pdfHeight = pdf.internal.pageSize.getHeight()
      const margin = EXPORT_CONFIG.pdf.margin
      
      const imgWidth = pdfWidth - 2 * margin
      const imgHeight = (canvas.height * imgWidth) / canvas.width
      
      // 添加标题
      pdf.setFontSize(16)
      pdf.text(title, margin, margin)
      
      // 添加图表
      const yPosition = margin + 10
      if (imgHeight <= pdfHeight - yPosition - margin) {
        pdf.addImage(imgData, 'PNG', margin, yPosition, imgWidth, imgHeight)
      } else {
        // 如果图表太高，缩放以适应页面
        const scaledHeight = pdfHeight - yPosition - margin
        const scaledWidth = (canvas.width * scaledHeight) / canvas.height
        pdf.addImage(imgData, 'PNG', margin, yPosition, scaledWidth, scaledHeight)
      }
      
      // 添加时间戳
      pdf.setFontSize(8)
      pdf.text(
        `导出时间: ${new Date().toLocaleString()}`,
        margin,
        pdfHeight - margin / 2
      )
      
      pdf.save(`${title}-${new Date().toISOString().split('T')[0]}.pdf`)
    } catch (error) {
      console.error('PDF导出失败:', error)
    } finally {
      setIsExporting(false)
    }
  }

  // 打印图表
  const printChart = () => {
    if (!chartRef.current) return

    const printWindow = window.open('', '_blank')
    if (!printWindow) return

    const chartHTML = chartRef.current.outerHTML
    
    printWindow.document.write(`
      <!DOCTYPE html>
      <html>
        <head>
          <title>${title} - 打印</title>
          <style>
            body {
              margin: 0;
              padding: 20px;
              font-family: 'Inter', sans-serif;
            }
            .chart-container {
              width: 100%;
              height: auto;
              display: flex;
              justify-content: center;
              align-items: center;
            }
            .header {
              text-align: center;
              margin-bottom: 20px;
            }
            .footer {
              text-align: center;
              margin-top: 20px;
              font-size: 12px;
              color: #666;
            }
            @media print {
              body { margin: 0; }
              .no-print { display: none; }
            }
          </style>
        </head>
        <body>
          <div class="header">
            <h2>${title}</h2>
          </div>
          <div class="chart-container">
            ${chartHTML}
          </div>
          <div class="footer">
            <p>导出时间: ${new Date().toLocaleString()}</p>
          </div>
        </body>
      </html>
    `)
    
    printWindow.document.close()
    printWindow.focus()
    
    // 等待内容加载后打印
    setTimeout(() => {
      printWindow.print()
      printWindow.close()
    }, 500)
  }

  const sizeClasses = {
    sm: 'h-8 w-8',
    md: 'h-9 w-9',
    lg: 'h-10 w-10'
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          className={`${sizeClasses[size]} ${className}`}
          disabled={isExporting}
          title="导出图表"
        >
          {isExporting ? (
            <div className="animate-spin h-4 w-4 border-2 border-current border-t-transparent rounded-full" />
          ) : (
            <Download className="h-4 w-4" />
          )}
        </Button>
      </DropdownMenuTrigger>
      
      <DropdownMenuContent align="end" className="w-48">
        <DropdownMenuItem onClick={exportToPNG} className="flex items-center gap-2">
          <Image className="h-4 w-4" />
          导出为 PNG
        </DropdownMenuItem>
        
        <DropdownMenuItem onClick={exportToSVG} className="flex items-center gap-2">
          <FileText className="h-4 w-4" />
          导出为 SVG
        </DropdownMenuItem>
        
        <DropdownMenuItem onClick={exportToPDF} className="flex items-center gap-2">
          <FileText className="h-4 w-4" />
          导出为 PDF
        </DropdownMenuItem>
        
        <DropdownMenuSeparator />
        
        <DropdownMenuItem onClick={printChart} className="flex items-center gap-2">
          <Printer className="h-4 w-4" />
          打印图表
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

// 批量导出组件
interface BatchExportProps {
  charts: Array<{
    ref: React.RefObject<any>
    title: string
  }>
  className?: string
}

export function BatchChartExport({ charts, className }: BatchExportProps) {
  const [isExporting, setIsExporting] = useState(false)

  const exportAllToPDF = async () => {
    if (charts.length === 0) return

    setIsExporting(true)
    try {
      const { default: jsPDF } = await import('jspdf')
      const { default: html2canvas } = await import('html2canvas')
      
      const pdf = new jsPDF({
        orientation: 'landscape',
        unit: 'mm',
        format: 'A4',
      })
      
      const pdfWidth = pdf.internal.pageSize.getWidth()
      const pdfHeight = pdf.internal.pageSize.getHeight()
      const margin = 20
      
      for (let i = 0; i < charts.length; i++) {
        const chart = charts[i]
        if (!chart.ref.current) continue
        
        if (i > 0) {
          pdf.addPage()
        }
        
        const canvas = await html2canvas(chart.ref.current, {
          backgroundColor: '#ffffff',
          scale: 1.5,
          useCORS: true,
          allowTaint: true,
        })
        
        const imgData = canvas.toDataURL('image/png')
        const imgWidth = pdfWidth - 2 * margin
        const imgHeight = (canvas.height * imgWidth) / canvas.width
        
        // 添加标题
        pdf.setFontSize(14)
        pdf.text(chart.title, margin, margin)
        
        // 添加图表
        const yPosition = margin + 10
        if (imgHeight <= pdfHeight - yPosition - margin) {
          pdf.addImage(imgData, 'PNG', margin, yPosition, imgWidth, imgHeight)
        } else {
          const scaledHeight = pdfHeight - yPosition - margin
          const scaledWidth = (canvas.width * scaledHeight) / canvas.height
          pdf.addImage(imgData, 'PNG', margin, yPosition, scaledWidth, scaledHeight)
        }
        
        // 添加页码
        pdf.setFontSize(8)
        pdf.text(
          `第 ${i + 1} 页，共 ${charts.length} 页`,
          pdfWidth - margin - 30,
          pdfHeight - margin / 2
        )
      }
      
      // 添加封面信息
      pdf.setFontSize(8)
      pdf.text(
        `导出时间: ${new Date().toLocaleString()}`,
        margin,
        pdfHeight - margin / 2
      )
      
      pdf.save(`charts-report-${new Date().toISOString().split('T')[0]}.pdf`)
    } catch (error) {
      console.error('批量导出失败:', error)
    } finally {
      setIsExporting(false)
    }
  }

  return (
    <Button
      variant="outline"
      onClick={exportAllToPDF}
      disabled={isExporting || charts.length === 0}
      className={`flex items-center gap-2 ${className}`}
    >
      {isExporting ? (
        <div className="animate-spin h-4 w-4 border-2 border-current border-t-transparent rounded-full" />
      ) : (
        <Download className="h-4 w-4" />
      )}
      导出所有图表
    </Button>
  )
}
