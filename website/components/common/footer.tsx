"use client"

import Link from 'next/link'
import { motion } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Database, Github, Twitter, Linkedin, Mail, MapPin, Phone } from 'lucide-react'
import { siteConfig, navConfig } from '@/config/site'
import { cn } from '@/lib/utils'

/**
 * 网站底部组件
 * 包含链接导航、公司信息、社交媒体链接等
 */
export function Footer() {
  const currentYear = new Date().getFullYear()

  const socialLinks = [
    {
      name: 'GitHub',
      href: siteConfig.links.github,
      icon: Github,
      description: '查看源代码'
    },
    {
      name: 'Twitter',
      href: siteConfig.links.twitter,
      icon: Twitter,
      description: '关注最新动态'
    },
    {
      name: 'LinkedIn',
      href: siteConfig.links.linkedin,
      icon: Linkedin,
      description: '商务合作'
    }
  ]

  const companyInfo = [
    {
      icon: MapPin,
      label: '地址',
      value: siteConfig.company.address
    },
    {
      icon: Mail,
      label: '邮箱',
      value: siteConfig.company.email
    },
    {
      icon: Phone,
      label: '电话',
      value: siteConfig.company.phone
    }
  ]

  return (
    <footer className="bg-background border-t border-border">
      <div className="container mx-auto px-4">
        {/* 主要内容区域 */}
        <div className="py-16">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-8">
            {/* 品牌信息 */}
            <div className="lg:col-span-2">
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                className="space-y-4"
              >
                <Link href="/" className="flex items-center space-x-2">
                  <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-green-500 to-blue-500 shadow-sm">
                    <Database className="h-5 w-5 text-white" />
                  </div>
                  <span className="text-xl font-bold bg-gradient-to-r from-green-600 to-blue-600 bg-clip-text text-transparent">
                    {siteConfig.name}
                  </span>
                </Link>
                
                <p className="text-muted-foreground leading-relaxed max-w-md">
                  {siteConfig.description}
                </p>
                
                {/* 社交媒体链接 */}
                <div className="flex space-x-4">
                  {socialLinks.map((social) => {
                    const Icon = social.icon
                    return (
                      <Button
                        key={social.name}
                        variant="ghost"
                        size="icon"
                        className="hover:bg-accent hover:text-accent-foreground transition-colors"
                        asChild
                      >
                        <Link href={social.href} target="_blank" rel="noopener noreferrer">
                          <Icon className="h-4 w-4" />
                          <span className="sr-only">{social.description}</span>
                        </Link>
                      </Button>
                    )
                  })}
                </div>
              </motion.div>
            </div>

            {/* 产品链接 */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.1 }}
            >
              <h3 className="font-semibold text-foreground mb-4">产品</h3>
              <ul className="space-y-3">
                {navConfig.footer.product.map((item) => (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className="text-muted-foreground hover:text-foreground transition-colors text-sm"
                    >
                      {item.title}
                    </Link>
                  </li>
                ))}
              </ul>
            </motion.div>

            {/* 文档链接 */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.2 }}
            >
              <h3 className="font-semibold text-foreground mb-4">文档</h3>
              <ul className="space-y-3">
                {navConfig.footer.docs.map((item) => (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className="text-muted-foreground hover:text-foreground transition-colors text-sm"
                    >
                      {item.title}
                    </Link>
                  </li>
                ))}
              </ul>
            </motion.div>

            {/* 公司信息 */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.3 }}
            >
              <h3 className="font-semibold text-foreground mb-4">公司</h3>
              <ul className="space-y-3">
                {navConfig.footer.company.map((item) => (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className="text-muted-foreground hover:text-foreground transition-colors text-sm"
                    >
                      {item.title}
                    </Link>
                  </li>
                ))}
              </ul>
            </motion.div>
          </div>
        </div>

        {/* 联系信息区域 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="py-8 border-t border-border"
        >
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {companyInfo.map((info, index) => {
              const Icon = info.icon
              return (
                <div key={index} className="flex items-center space-x-3">
                  <div className="w-8 h-8 rounded-lg bg-muted flex items-center justify-center">
                    <Icon className="w-4 h-4 text-muted-foreground" />
                  </div>
                  <div>
                    <div className="text-xs text-muted-foreground">{info.label}</div>
                    <div className="text-sm font-medium text-foreground">{info.value}</div>
                  </div>
                </div>
              )
            })}
          </div>
        </motion.div>

        <Separator />

        {/* 底部版权信息 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="py-6"
        >
          <div className="flex flex-col md:flex-row justify-between items-center space-y-4 md:space-y-0">
            <div className="text-sm text-muted-foreground">
              © {currentYear} {siteConfig.company.name}. 保留所有权利。
            </div>
            
            <div className="flex items-center space-x-6">
              {navConfig.footer.legal.map((item, index) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  {item.title}
                </Link>
              ))}
            </div>
          </div>
        </motion.div>

        {/* 技术标识 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="py-4 border-t border-border"
        >
          <div className="text-center">
            <p className="text-xs text-muted-foreground">
              Built with ❤️ using{' '}
              <span className="font-medium">Next.js</span>,{' '}
              <span className="font-medium">React</span>,{' '}
              <span className="font-medium">TypeScript</span>,{' '}
              <span className="font-medium">Tailwind CSS</span> &{' '}
              <span className="font-medium">shadcn/ui</span>
            </p>
          </div>
        </motion.div>
      </div>
    </footer>
  )
}