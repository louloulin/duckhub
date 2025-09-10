"use client"

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { motion, AnimatePresence } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { Database, Menu, X, ChevronDown, ExternalLink } from 'lucide-react'
import { cn } from '@/lib/utils'
import { siteConfig, navConfig } from '@/config/site'

/**
 * 网站头部导航组件
 * 包含Logo、导航菜单、CTA按钮和移动端菜单
 */
export function Header() {
  const [isScrolled, setIsScrolled] = useState(false)
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)
  const [activeDropdown, setActiveDropdown] = useState<string | null>(null)
  const pathname = usePathname()

  // 监听滚动状态
  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 10)
    }
    
    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  // 关闭下拉菜单
  useEffect(() => {
    const handleClickOutside = () => {
      setActiveDropdown(null)
    }
    
    document.addEventListener('click', handleClickOutside)
    return () => document.removeEventListener('click', handleClickOutside)
  }, [])

  const handleDropdownToggle = (dropdown: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setActiveDropdown(activeDropdown === dropdown ? null : dropdown)
  }

  return (
    <header className={cn(
      "sticky top-0 z-50 w-full border-b transition-all duration-200",
      isScrolled 
        ? "bg-background/80 backdrop-blur-md border-border shadow-sm" 
        : "bg-background/50 border-transparent"
    )}>
      <div className="container flex h-16 items-center justify-between">
        {/* Logo */}
        <Link href="/" className="flex items-center space-x-2 hover:opacity-80 transition-opacity">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-green-500 to-blue-500 shadow-sm">
            <Database className="h-5 w-5 text-white" />
          </div>
          <span className="text-xl font-bold bg-gradient-to-r from-green-600 to-blue-600 bg-clip-text text-transparent">
            {siteConfig.name}
          </span>
        </Link>

        {/* 桌面端导航 */}
        <nav className="hidden lg:flex items-center space-x-1">
          {navConfig.main.map((item) => {
            const isActive = pathname === item.href || pathname.startsWith(item.href + '/')
            
            if (item.href === '/features') {
              return (
                <div key={item.href} className="relative">
                  <Button
                    variant="ghost"
                    className={cn(
                      "h-9 px-3 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
                      isActive && "bg-accent text-accent-foreground"
                    )}
                    onClick={(e) => handleDropdownToggle('features', e)}
                  >
                    {item.title}
                    <ChevronDown className="ml-1 h-3 w-3" />
                  </Button>
                  
                  <AnimatePresence>
                    {activeDropdown === 'features' && (
                      <motion.div
                        initial={{ opacity: 0, y: -10 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -10 }}
                        transition={{ duration: 0.2 }}
                        className="absolute top-full left-0 mt-1 w-80 bg-popover border border-border rounded-lg shadow-lg p-4"
                      >
                        <div className="grid gap-3">
                          {navConfig.features.map((feature) => (
                            <Link
                              key={feature.href}
                              href={feature.href}
                              className="block p-3 rounded-md hover:bg-accent transition-colors"
                              onClick={() => setActiveDropdown(null)}
                            >
                              <div className="flex items-start space-x-3">
                                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-green-500 to-blue-500 flex items-center justify-center flex-shrink-0">
                                  <Database className="w-4 h-4 text-white" />
                                </div>
                                <div>
                                  <div className="font-medium text-sm">{feature.title}</div>
                                  <div className="text-xs text-muted-foreground mt-1">
                                    {feature.description}
                                  </div>
                                </div>
                              </div>
                            </Link>
                          ))}
                        </div>
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              )
            }
            
            if (item.href === '/docs') {
              return (
                <div key={item.href} className="relative">
                  <Button
                    variant="ghost"
                    className={cn(
                      "h-9 px-3 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
                      isActive && "bg-accent text-accent-foreground"
                    )}
                    onClick={(e) => handleDropdownToggle('docs', e)}
                  >
                    {item.title}
                    <ChevronDown className="ml-1 h-3 w-3" />
                  </Button>
                  
                  <AnimatePresence>
                    {activeDropdown === 'docs' && (
                      <motion.div
                        initial={{ opacity: 0, y: -10 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -10 }}
                        transition={{ duration: 0.2 }}
                        className="absolute top-full left-0 mt-1 w-64 bg-popover border border-border rounded-lg shadow-lg p-4"
                      >
                        <div className="space-y-2">
                          {navConfig.docs.map((doc) => (
                            <Link
                              key={doc.href}
                              href={doc.href}
                              className="block p-2 rounded-md hover:bg-accent transition-colors"
                              onClick={() => setActiveDropdown(null)}
                            >
                              <div className="font-medium text-sm">{doc.title}</div>
                              <div className="text-xs text-muted-foreground mt-1">
                                {doc.description}
                              </div>
                            </Link>
                          ))}
                        </div>
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              )
            }
            
            return (
              <Button
                key={item.href}
                variant="ghost"
                className={cn(
                  "h-9 px-3 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
                  isActive && "bg-accent text-accent-foreground"
                )}
                asChild
              >
                <Link href={item.href}>{item.title}</Link>
              </Button>
            )
          })}
        </nav>

        {/* CTA按钮 */}
        <div className="hidden lg:flex items-center space-x-2">
          <Button variant="ghost" asChild>
            <Link href="/docs">文档</Link>
          </Button>
          <Button className="bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700" asChild>
            <Link href="/contact" className="flex items-center space-x-1">
              <span>开始使用</span>
              <ExternalLink className="h-3 w-3" />
            </Link>
          </Button>
        </div>

        {/* 移动端菜单 */}
        <Sheet open={isMobileMenuOpen} onOpenChange={setIsMobileMenuOpen}>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon" className="lg:hidden">
              <Menu className="h-5 w-5" />
              <span className="sr-only">切换菜单</span>
            </Button>
          </SheetTrigger>
          <SheetContent side="right" className="w-[300px] sm:w-[400px]">
            <div className="flex flex-col space-y-4">
              <div className="flex items-center space-x-2">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-green-500 to-blue-500">
                  <Database className="h-5 w-5 text-white" />
                </div>
                <span className="text-xl font-bold bg-gradient-to-r from-green-600 to-blue-600 bg-clip-text text-transparent">
                  {siteConfig.name}
                </span>
              </div>
              
              <nav className="flex flex-col space-y-2">
                {navConfig.main.map((item) => (
                  <div key={item.href}>
                    <Link
                      href={item.href}
                      className="block px-2 py-1 text-lg font-medium hover:text-foreground/80 transition-colors"
                      onClick={() => setIsMobileMenuOpen(false)}
                    >
                      {item.title}
                    </Link>
                    {item.href === '/features' && (
                      <div className="ml-4 mt-2 space-y-2">
                        {navConfig.features.map((feature) => (
                          <Link
                            key={feature.href}
                            href={feature.href}
                            className="block px-2 py-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
                            onClick={() => setIsMobileMenuOpen(false)}
                          >
                            {feature.title}
                          </Link>
                        ))}
                      </div>
                    )}
                    {item.href === '/docs' && (
                      <div className="ml-4 mt-2 space-y-2">
                        {navConfig.docs.map((doc) => (
                          <Link
                            key={doc.href}
                            href={doc.href}
                            className="block px-2 py-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
                            onClick={() => setIsMobileMenuOpen(false)}
                          >
                            {doc.title}
                          </Link>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </nav>
              
              <div className="flex flex-col space-y-2 pt-4 border-t">
                <Button variant="ghost" asChild>
                  <Link href="/docs" onClick={() => setIsMobileMenuOpen(false)}>
                    文档
                  </Link>
                </Button>
                <Button className="bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700" asChild>
                  <Link href="/contact" onClick={() => setIsMobileMenuOpen(false)} className="flex items-center justify-center space-x-1">
                    <span>开始使用</span>
                    <ExternalLink className="h-3 w-3" />
                  </Link>
                </Button>
              </div>
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </header>
  )
}